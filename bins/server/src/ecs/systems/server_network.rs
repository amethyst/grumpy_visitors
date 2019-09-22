use amethyst::ecs::{Join, ReadExpect, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::{MultiplayerGameState, MultiplayerRoomPlayer},
            world::{FramedUpdates, PlayerActionUpdates},
            GameEngineState, NewGameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload,
        ConnectionIdentifier, NetConnection, NetEvent,
    },
};
use ha_game::{
    ecs::resources::ConnectionEvents,
    utils::net::{broadcast_message_reliable, send_message_reliable},
};

// Pause the game if we have a client that hasn't responded for the last 180 frames (3 secs).
const PAUSE_FRAME_THRESHOLD: u64 = 180;

pub struct ServerNetworkSystem {
    host_connection_id: ConnectionIdentifier,
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
                        update.add_cast_action_update(action);
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
                    connection_model.last_acknowledged_update = Some(frame_number);
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
            multiplayer_game_state.waiting_network = false;
            let min_last_acknowledged_update = (&net_connection_models)
                .join()
                .map(|net_connection_model| net_connection_model.last_acknowledged_update)
                .min_by(|update_a, update_b| update_a.cmp(update_b));
            if let Some(min_last_acknowledged_update) = min_last_acknowledged_update {
                let frames_ahead = min_last_acknowledged_update
                    .map(|update_number| {
                        game_time_service
                            .game_frame_number()
                            .saturating_sub(update_number)
                    })
                    .unwrap_or_else(|| game_time_service.game_frame_number() + 1);
                log::trace!("Frames ahead: {}", frames_ahead);
                if frames_ahead > PAUSE_FRAME_THRESHOLD {
                    multiplayer_game_state.waiting_network = true;
                }
            }
        }
    }
}
