use amethyst::ecs::{Join, ReadExpect, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::{MultiplayerGameState, MultiplayerRoomPlayer},
            world::{FramedUpdates, PlayerActionUpdates, LAG_COMPENSATION_FRAMES_LIMIT},
            GameEngineState, NewGameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload, NetConnection,
        NetEvent, NetIdentifier, INTERPOLATION_FRAME_DELAY,
    },
};
use ha_game::{
    ecs::resources::ConnectionEvents,
    utils::net::{broadcast_message_reliable, send_message_reliable},
};

// Pause the game if we have a client that hasn't responded for the last 180 frames (3 secs).
const PAUSE_FRAME_THRESHOLD: u64 =
    (LAG_COMPENSATION_FRAMES_LIMIT + LAG_COMPENSATION_FRAMES_LIMIT / 2) as u64;

pub struct ServerNetworkSystem {
    host_connection_id: NetIdentifier,
}

impl ServerNetworkSystem {
    pub fn new() -> Self {
        Self {
            host_connection_id: 0,
        }
    }
}

impl<'s> System<'s> for ServerNetworkSystem {
    type SystemData = (
        GameTimeService<'s>,
        ReadExpect<'s, GameEngineState>,
        WriteExpect<'s, ConnectionEvents>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, NewGameEngineState>,
        WriteExpect<'s, FramedUpdates<PlayerActionUpdates>>,
        WriteStorage<'s, NetConnection>,
        WriteStorage<'s, NetConnectionModel>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_engine_state,
            mut connection_events,
            mut multiplayer_game_state,
            mut new_game_engine_state,
            mut framed_updates,
            mut net_connections,
            mut net_connection_models,
        ): Self::SystemData,
    ) {
        for connection_event in connection_events.0.drain(..) {
            let connection_id = connection_event.connection_id;
            match connection_event.event {
                NetEvent::Connected => {
                    // TODO: we'll need a more reliable way to determine the host in future.
                    if multiplayer_game_state.players.is_empty() {
                        self.host_connection_id = connection_id;
                    }

                    log::info!("Sending a Handshake message: {}", connection_id);
                    let (net_connection, _) = (&mut net_connections, &net_connection_models)
                        .join()
                        .find(|(_, net_connection_model)| net_connection_model.id == connection_id)
                        .expect("Expected to find a NetConnection");
                    send_message_reliable(
                        net_connection,
                        &ServerMessagePayload::Handshake(connection_id),
                    );
                }
                NetEvent::Message(ClientMessagePayload::JoinRoom { nickname }) => {
                    multiplayer_game_state
                        .update_players()
                        .push(MultiplayerRoomPlayer {
                            connection_id,
                            entity_net_id: 0,
                            nickname,
                            is_host: self.host_connection_id == connection_id,
                        });
                }
                NetEvent::Message(ClientMessagePayload::StartHostedGame)
                    if connection_id == self.host_connection_id =>
                {
                    multiplayer_game_state.is_playing = true;
                    new_game_engine_state.0 = GameEngineState::Playing;
                }
                NetEvent::Message(ClientMessagePayload::WalkActions(mut action)) => {
                    if let Some(update) = framed_updates.update_frame(action.frame_number, true) {
                        log::info!(
                            "Added an update for frame {} to frame {}",
                            action.frame_number,
                            update.frame_number
                        );
                        action.frame_number = update.frame_number;
                        update.add_walk_action_updates(action);
                    }
                }
                NetEvent::Message(ClientMessagePayload::CastActions(mut action)) => {
                    if let Some(update) = framed_updates.update_frame(action.frame_number, true) {
                        action.frame_number = update.frame_number;
                        update.add_cast_action_updates(action);
                    }
                }
                NetEvent::Message(ClientMessagePayload::AcknowledgeWorldUpdate(frame_number)) => {
                    let mut connection_model = (&mut net_connection_models)
                        .join()
                        .find(|model| model.id == connection_id)
                        .unwrap_or_else(|| {
                            panic!(
                                "Expected to find a connection model with id {}",
                                connection_id
                            )
                        });
                    connection_model.last_acknowledged_update =
                        Some(frame_number).max(connection_model.last_acknowledged_update);
                }
                NetEvent::Disconnected => {
                    multiplayer_game_state
                        .update_players()
                        .retain(|player| player.connection_id == connection_id);
                }
                _ => {}
            }
        }

        if let Some(players) = multiplayer_game_state.read_updated_players() {
            broadcast_message_reliable(
                &mut net_connections,
                &ServerMessagePayload::UpdateRoomPlayers(players.to_owned()),
            );
        }

        // Pause server if one of clients is lagging behind.
        if *game_engine_state == GameEngineState::Playing && multiplayer_game_state.is_playing {
            let mut lagging_players = Vec::new();
            for net_connection_model in (&net_connection_models).join() {
                let frames_since_last_pong = game_time_service.engine_time().frame_number()
                    - net_connection_model.ping_pong_data.last_ponged_frame;
                let average_lagging_behind =
                    net_connection_model.ping_pong_data.average_lagging_behind();

                let expected_client_frame_number = game_time_service
                    .game_frame_number()
                    .saturating_sub(INTERPOLATION_FRAME_DELAY);

                let was_lagging = multiplayer_game_state
                    .lagging_players
                    .iter()
                    .any(|connection_id| *connection_id == net_connection_model.id);

                // If a player was already lagging we expect them to fully catch up with others.
                let is_catching_up = net_connection_model.ping_pong_data.last_stored_game_frame()
                    < expected_client_frame_number;

                log::trace!(
                    "Average lagging behind (client {}): {}",
                    net_connection_model.id,
                    average_lagging_behind
                );

                if frames_since_last_pong > PAUSE_FRAME_THRESHOLD
                    || was_lagging && is_catching_up
                    || average_lagging_behind > PAUSE_FRAME_THRESHOLD
                {
                    lagging_players.push(net_connection_model.id);
                }
            }

            multiplayer_game_state.lagging_players = lagging_players.clone();
            if !multiplayer_game_state.waiting_for_players && !lagging_players.is_empty() {
                multiplayer_game_state.waiting_for_players_pause_id += 1;
                broadcast_message_reliable(
                    &mut net_connections,
                    &ServerMessagePayload::PauseWaitingForPlayers {
                        id: multiplayer_game_state.waiting_for_players_pause_id,
                        players: lagging_players,
                    },
                );
                multiplayer_game_state.waiting_for_players = true;
            } else if multiplayer_game_state.waiting_for_players && lagging_players.is_empty() {
                broadcast_message_reliable(
                    &mut net_connections,
                    &ServerMessagePayload::UnpauseWaitingForPlayers(
                        multiplayer_game_state.waiting_for_players_pause_id,
                    ),
                );
                multiplayer_game_state.waiting_for_players = false;
            }
        }
    }
}
