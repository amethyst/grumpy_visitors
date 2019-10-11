use amethyst::ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use std::cmp::Ordering;

use ha_client_shared::ecs::resources::MultiplayerRoomState;
use ha_core::{
    actions::monster_spawn::SpawnActions,
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::MultiplayerGameState,
            world::{
                FramedUpdates, PlayerActionUpdates, ReceivedPlayerUpdate,
                ReceivedServerWorldUpdate, ServerWorldUpdate, LAG_COMPENSATION_FRAMES_LIMIT,
            },
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
    utils::net::{send_message_reliable, send_message_unreliable},
};

use crate::ecs::resources::LastAcknowledgedUpdate;

// Pause the game if we haven't received any message from server for the last 180 frames (3 secs).
const PAUSE_FRAME_THRESHOLD: u64 =
    (LAG_COMPENSATION_FRAMES_LIMIT + LAG_COMPENSATION_FRAMES_LIMIT / 2) as u64;

pub struct ClientNetworkSystem;

impl<'s> System<'s> for ClientNetworkSystem {
    type SystemData = (
        GameTimeService<'s>,
        ReadExpect<'s, GameEngineState>,
        Entities<'s>,
        WriteExpect<'s, ConnectionEvents>,
        WriteExpect<'s, MultiplayerRoomState>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, NewGameEngineState>,
        WriteExpect<'s, LastAcknowledgedUpdate>,
        WriteExpect<'s, FramedUpdates<ReceivedServerWorldUpdate>>,
        WriteExpect<'s, FramedUpdates<PlayerActionUpdates>>,
        WriteExpect<'s, FramedUpdates<SpawnActions>>,
        WriteStorage<'s, NetConnection>,
        ReadStorage<'s, NetConnectionModel>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_engine_state,
            entities,
            mut connection_events,
            mut multiplayer_room_state,
            mut multiplayer_game_state,
            mut new_game_engine_sate,
            mut last_acknowledged_update,
            mut framed_updates,
            mut player_actions_updates,
            mut spawn_actions,
            mut connections,
            net_connection_models,
        ): Self::SystemData,
    ) {
        if connections.count() == 0
            && multiplayer_room_state.is_active
            && !multiplayer_room_state.is_host
        {
            entities
                .build_entity()
                .with(
                    NetConnection::new(multiplayer_room_state.server_addr),
                    &mut connections,
                )
                .build();
            return;
        }

        if multiplayer_room_state.is_host
            && multiplayer_room_state.has_started
            && !multiplayer_room_state.has_sent_start_package
        {
            multiplayer_room_state.has_sent_start_package = true;
            let connection = (&mut connections)
                .join()
                .next()
                .expect("Expected a server connection");
            send_message_reliable(connection, &ClientMessagePayload::StartHostedGame);
        }

        for connection_event in connection_events.0.drain(..) {
            match connection_event.event {
                NetEvent::Message(ServerMessagePayload::Handshake(connection_id)) => {
                    log::info!("Received Handshake from a server ({})", connection_id);
                    let connection = (&mut connections)
                        .join()
                        .next()
                        .expect("Expected a server connection");
                    multiplayer_room_state.connection_id = connection_id;

                    send_message_reliable(
                        connection,
                        &ClientMessagePayload::JoinRoom {
                            nickname: multiplayer_room_state.nickname.clone(),
                        },
                    );
                }
                NetEvent::Message(ServerMessagePayload::UpdateRoomPlayers(players)) => {
                    log::info!("Updated room players");
                    *multiplayer_game_state.update_players() = players;
                }
                NetEvent::Message(ServerMessagePayload::StartGame(entity_net_ids)) => {
                    for (i, player) in multiplayer_game_state
                        .update_players()
                        .iter_mut()
                        .enumerate()
                    {
                        player.entity_net_id = entity_net_ids[i];
                        if player.connection_id == multiplayer_room_state.connection_id {
                            multiplayer_room_state.player_net_id = player.entity_net_id;
                        }
                    }
                    multiplayer_game_state.is_playing = true;
                    new_game_engine_sate.0 = GameEngineState::Playing;
                }
                NetEvent::Message(ServerMessagePayload::UpdateWorld { id, mut updates }) => {
                    let connection = (&mut connections)
                        .join()
                        .next()
                        .expect("Expected a server connection");
                    send_message_unreliable(
                        connection,
                        &ClientMessagePayload::AcknowledgeWorldUpdate(id),
                    );

                    if last_acknowledged_update.id < id {
                        updates.sort_by(|a, b| a.frame_number.cmp(&b.frame_number));

                        last_acknowledged_update.id = id;
                        last_acknowledged_update.frame_number =
                            last_acknowledged_update.frame_number.max(
                                updates
                                    .last()
                                    .map(|update| update.frame_number)
                                    .unwrap_or(0),
                            );

                        let frame_to_reserve = last_acknowledged_update
                            .frame_number
                            .max(game_time_service.game_frame_number());
                        framed_updates.reserve_updates(frame_to_reserve);
                        spawn_actions.reserve_updates(frame_to_reserve);

                        apply_world_updates(
                            vec![multiplayer_room_state.player_net_id],
                            &mut framed_updates,
                            &mut spawn_actions,
                            updates,
                        );
                    }
                }
                NetEvent::Message(ServerMessagePayload::DiscardWalkActions(discarded_actions)) => {
                    discard_walk_actions(&mut player_actions_updates, discarded_actions);
                }
                NetEvent::Message(ServerMessagePayload::PauseWaitingForPlayers { id, players }) => {
                    if multiplayer_game_state.waiting_for_players_pause_id < id {
                        // We don't always want set `waiting_for_players` to true, as we may need
                        // to catch up with the server if we're lagging too. See below.
                        multiplayer_game_state.waiting_for_players_pause_id = id;
                        multiplayer_game_state.lagging_players = players;
                    }
                }
                NetEvent::Message(ServerMessagePayload::UnpauseWaitingForPlayers(id)) => {
                    if multiplayer_game_state.waiting_for_players_pause_id <= id {
                        multiplayer_game_state.waiting_for_players = false;
                        multiplayer_game_state.waiting_for_players_pause_id = id;
                        multiplayer_game_state.lagging_players.clear();
                    }
                }
                // TODO: handle disconnects.
                _ => {}
            }
        }

        // Until the server authorizes to unpause we need to use a chance to catch up with it,
        // even if it's not us lagging.
        if !multiplayer_game_state.lagging_players.is_empty() {
            let server_frame = framed_updates
                .updates
                .back()
                .expect("Expected at least one framed update if we're lagging")
                .frame_number;

            multiplayer_game_state.waiting_for_players =
                game_time_service.game_frame_number() + INTERPOLATION_FRAME_DELAY >= server_frame;
        }

        if *game_engine_state == GameEngineState::Playing && multiplayer_game_state.is_playing {
            // We always skip first INTERPOLATION_FRAME_DELAY frames on game start.
            match game_time_service
                .game_frame_number_absolute()
                .cmp(&INTERPOLATION_FRAME_DELAY)
            {
                Ordering::Less => {
                    multiplayer_game_state.waiting_network = true;
                    return;
                }
                Ordering::Equal => {
                    multiplayer_game_state.waiting_network = false;
                }
                _ => {}
            }

            // Wait if we a server is lagging behind for PAUSE_FRAME_THRESHOLD frames.
            let frames_ahead = game_time_service.game_frame_number().saturating_sub(
                last_acknowledged_update
                    .frame_number
                    .saturating_sub(INTERPOLATION_FRAME_DELAY),
            );
            log::trace!("Frames ahead: {}", frames_ahead);
            if multiplayer_game_state.waiting_network {
                multiplayer_game_state.waiting_network = frames_ahead == 0;
            } else if frames_ahead > PAUSE_FRAME_THRESHOLD {
                multiplayer_game_state.waiting_network = true;
            }

            if multiplayer_game_state.waiting_network || multiplayer_game_state.waiting_for_players
            {
                let net_connection_model = (&net_connection_models)
                    .join()
                    .next()
                    .expect("Expected a server connection (NetConnectionModel)");

                log::debug!(
                    "Waiting for server. Frames ahead: {}. Current frame: {}. Last ServerWorldUpdate frame: {}. Estimated server frame: {}",
                    frames_ahead,
                    game_time_service.game_frame_number(),
                    last_acknowledged_update.frame_number,
                    net_connection_model.ping_pong_data.last_stored_game_frame(),
                );
            }
        }
    }
}

// Expects incoming_updates to be sorted (lowest frame first).
fn apply_world_updates(
    controlled_players: Vec<NetIdentifier>,
    framed_updates: &mut FramedUpdates<ReceivedServerWorldUpdate>,
    spawn_actions: &mut FramedUpdates<SpawnActions>,
    mut incoming_updates: Vec<ServerWorldUpdate>,
) {
    if incoming_updates.is_empty() {
        return;
    }

    let first_incoming_frame_number = incoming_updates
        .first()
        .unwrap()
        .frame_number
        .saturating_sub(INTERPOLATION_FRAME_DELAY);
    let first_available_frame_number = framed_updates.updates.front().unwrap().frame_number;
    assert!(
        first_incoming_frame_number >= first_available_frame_number,
        "Tried to apply a too old ServerUpdate (frame {}), when the first available frame is {}",
        first_incoming_frame_number,
        first_available_frame_number,
    );

    let controlled_player_updates =
        collect_controlled_player_updates(&controlled_players, &mut incoming_updates);

    let (controlled_start_frame_number, others_start_frame_number) = incoming_updates
        .first()
        .map(|update| {
            (
                update
                    .frame_number
                    .saturating_sub(INTERPOLATION_FRAME_DELAY),
                update.frame_number,
            )
        })
        .unwrap();

    spawn_actions.oldest_updated_frame = others_start_frame_number;
    for (spawn_actions, server_update) in spawn_actions
        .updates_iter_mut(others_start_frame_number)
        .zip(incoming_updates.iter())
    {
        spawn_actions.spawn_actions = server_update.spawn_actions.clone()
    }

    framed_updates.oldest_updated_frame = controlled_start_frame_number;
    let mut controlled_player_updates_iter = controlled_player_updates.into_iter();
    let mut incoming_updates_iter = incoming_updates.into_iter();

    for frame_updates in framed_updates.updates_iter_mut(controlled_start_frame_number) {
        if let Some(controlled_player_updates) = controlled_player_updates_iter.next() {
            frame_updates.controlled_player_updates = controlled_player_updates;
        }
        if frame_updates.frame_number >= others_start_frame_number {
            let server_update = incoming_updates_iter.next();
            if server_update.is_none() {
                return;
            }
            frame_updates.apply_server_update(server_update.unwrap());
        }
    }
}

fn collect_controlled_player_updates(
    controlled_players: &[NetIdentifier],
    incoming_updates: &mut Vec<ServerWorldUpdate>,
) -> Vec<ReceivedPlayerUpdate> {
    incoming_updates
        .iter_mut()
        .skip_while(|update| {
            // Skips the first 10 frames, as there shouldn't be any player updates on game start.
            update.frame_number < INTERPOLATION_FRAME_DELAY
        })
        .map(|update| {
            let mut controlled_player_update = ReceivedPlayerUpdate::default();

            let walk_action_pos = update
                .player_walk_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(walk_action_pos) = walk_action_pos {
                let walk_action = update.player_walk_actions_updates.remove(walk_action_pos);
                controlled_player_update
                    .player_walk_actions_updates
                    .push(walk_action);
            }

            let cast_action_pos = update
                .player_cast_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(cast_action_pos) = cast_action_pos {
                let cast_action = update.player_cast_actions_updates.remove(cast_action_pos);
                controlled_player_update
                    .player_cast_actions_updates
                    .push(cast_action);
            }

            let look_action_pos = update
                .player_look_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(look_action_pos) = look_action_pos {
                // We just remove a look action here, as we are not interested in replaying it.
                update.player_look_actions_updates.remove(look_action_pos);
            }

            controlled_player_update
        })
        .collect()
}

fn discard_walk_actions(
    client_player_updates: &mut FramedUpdates<PlayerActionUpdates>,
    mut discarded_updates: Vec<NetIdentifier>,
) {
    let mut oldest_updated_frame = client_player_updates.oldest_updated_frame;
    for update in client_player_updates.updates.iter_mut().rev() {
        let update_frame_number = update.frame_number;
        update.walk_action_updates.retain(|net_update| {
            if let Some(i) = discarded_updates
                .iter()
                .position(|discarded_update| *discarded_update == net_update.data.client_action_id)
            {
                discarded_updates.remove(i);
                oldest_updated_frame = update_frame_number;
                false
            } else {
                true
            }
        });

        if discarded_updates.is_empty() {
            break;
        }
    }
    client_player_updates.oldest_updated_frame = oldest_updated_frame;
}
