use amethyst::ecs::{Entities, Join, ReadExpect, System, WriteExpect, WriteStorage};

use std::{collections::VecDeque, iter::FromIterator};

use ha_client_shared::ecs::resources::MultiplayerRoomState;
use ha_core::{
    ecs::{
        resources::{
            net::MultiplayerGameState,
            world::{FramedUpdate, FramedUpdates, ServerWorldUpdate},
            GameEngineState, NewGameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload, NetConnection,
        NetEvent, NetIdentifier,
    },
};
use ha_game::{
    ecs::resources::ConnectionEvents,
    utils::net::{send_message_reliable, send_message_unreliable},
};

use crate::ecs::resources::LastAcknowledgedUpdate;

pub const INTERPOLATION_FRAME_DELAY: u64 = 10;

// Pause the game if we haven't received any message from server for the last 180 frames (3 secs).
const PAUSE_FRAME_THRESHOLD: u64 = 180;

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
        WriteExpect<'s, FramedUpdates<ServerWorldUpdate>>,
        WriteStorage<'s, NetConnection>,
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
            mut connections,
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

                    updates.sort_by(|a, b| a.frame_number.cmp(&b.frame_number));
                    last_acknowledged_update.0 = updates
                        .last()
                        .expect("Expected at least one incoming server update")
                        .frame_number;
                    apply_world_updates(
                        multiplayer_room_state.player_net_id,
                        &mut framed_updates,
                        updates,
                    );
                }
                // TODO: handle disconnects.
                _ => {}
            }
        }

        if *game_engine_state == GameEngineState::Playing && multiplayer_game_state.is_playing {
            multiplayer_game_state.waiting_network = false;
            if game_time_service.game_frame_number_absolute() < INTERPOLATION_FRAME_DELAY {
                multiplayer_game_state.waiting_network = true;
                return;
            }

            let frames_ahead = game_time_service.game_frame_number().saturating_sub(
                last_acknowledged_update
                    .0
                    .saturating_sub(INTERPOLATION_FRAME_DELAY),
            );
            log::trace!("Frames ahead: {}", frames_ahead);
            multiplayer_game_state.waiting_network = frames_ahead > PAUSE_FRAME_THRESHOLD;
        }
    }
}

// Expects incoming_updates to be sorted (lowest frame first).
fn apply_world_updates(
    player_net_id: NetIdentifier,
    framed_updates: &mut FramedUpdates<ServerWorldUpdate>,
    mut incoming_updates: Vec<ServerWorldUpdate>,
) {
    let current_player_updates = incoming_updates
        .iter_mut()
        .skip_while(|update| update.frame_number < INTERPOLATION_FRAME_DELAY)
        .map(|update| {
            let mut current_player_update =
                ServerWorldUpdate::new_update(update.frame_number - INTERPOLATION_FRAME_DELAY);

            let walk_action_pos = update
                .player_walk_actions_updates
                .iter()
                .position(|action| action.entity_net_id == player_net_id);
            if let Some(walk_action_pos) = walk_action_pos {
                let walk_action = update.player_walk_actions_updates.remove(walk_action_pos);
                current_player_update
                    .player_walk_actions_updates
                    .push(walk_action);
            }

            let cast_action_pos = update
                .player_cast_actions_updates
                .iter()
                .position(|action| action.entity_net_id == player_net_id);
            if let Some(cast_action_pos) = cast_action_pos {
                let cast_action = update.player_cast_actions_updates.remove(cast_action_pos);
                current_player_update
                    .player_cast_actions_updates
                    .push(cast_action);
            }

            let look_action_pos = update
                .player_look_actions_updates
                .iter()
                .position(|action| action.entity_net_id == player_net_id);
            if let Some(look_action_pos) = look_action_pos {
                let look_action = update.player_look_actions_updates.remove(look_action_pos);
                current_player_update
                    .player_look_actions_updates
                    .push(look_action);
            }

            current_player_update
        })
        .collect();
    apply_filtered_world_updates(framed_updates, incoming_updates);
    apply_filtered_world_updates(framed_updates, current_player_updates);
}

fn apply_filtered_world_updates(
    framed_updates: &mut FramedUpdates<ServerWorldUpdate>,
    incoming_updates: Vec<ServerWorldUpdate>,
) {
    let start_frame_number = incoming_updates.first().map(|update| update.frame_number);
    if let Some(start_frame_number) = start_frame_number {
        framed_updates.oldest_updated_frame = start_frame_number;
        let mut incoming_updates_iter = incoming_updates.into_iter();
        let mut frame_number = start_frame_number;

        // There may be updates altering the old ones, merge them if such exist.
        for frame_updates in framed_updates.updates_iter_mut(start_frame_number) {
            let server_update = incoming_updates_iter.next();
            if server_update.is_none() {
                return;
            }
            let server_update = server_update.unwrap();
            frame_number += 1;
            frame_updates.merge_another_update(server_update);
        }

        // Add all the other updates.
        if let Some(newest_update) = framed_updates.updates.back() {
            assert_eq!(newest_update.frame_number + 1, frame_number);
        }
        framed_updates
            .updates
            .append(&mut VecDeque::from_iter(incoming_updates_iter));
    }
}
