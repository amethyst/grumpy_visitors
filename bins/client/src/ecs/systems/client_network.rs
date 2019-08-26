use amethyst::ecs::{Entities, Join, System, WriteExpect, WriteStorage};

use std::{collections::VecDeque, iter::FromIterator};

use ha_client_shared::ecs::resources::MultiplayerRoomState;
use ha_core::{
    ecs::resources::{
        net::MultiplayerGameState,
        world::{FramedUpdates, ServerWorldUpdate},
        GameEngineState, NewGameEngineState,
    },
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload, NetConnection,
        NetEvent,
    },
};
use ha_game::{ecs::resources::ConnectionEvents, utils::net::send_message_reliable};

pub struct ClientNetworkSystem;

impl<'s> System<'s> for ClientNetworkSystem {
    type SystemData = (
        Entities<'s>,
        WriteExpect<'s, ConnectionEvents>,
        WriteExpect<'s, MultiplayerRoomState>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, NewGameEngineState>,
        WriteExpect<'s, FramedUpdates<ServerWorldUpdate>>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut connection_events,
            mut multiplayer_room_state,
            mut multiplayer_game_state,
            mut new_game_engine_sate,
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
                NetEvent::Message(ServerMessagePayload::UpdateRoomPlayers(players)) => {
                    log::info!("Updated room players");
                    *multiplayer_game_state.update_players() = players;
                }
                NetEvent::Message(ServerMessagePayload::StartGame(entity_net_identifiers)) => {
                    for (i, player) in multiplayer_game_state
                        .update_players()
                        .iter_mut()
                        .enumerate()
                    {
                        player.entity_net_id = entity_net_identifiers[i];
                    }
                    multiplayer_game_state.is_playing = true;
                    new_game_engine_sate.0 = GameEngineState::Playing;
                }
                NetEvent::Message(ServerMessagePayload::UpdateWorld { id, updates }) => {
                    let connection = (&mut connections)
                        .join()
                        .next()
                        .expect("Expected a server connection");
                    send_message_reliable(
                        connection,
                        &ClientMessagePayload::AcknowledgeWorldUpdate(id),
                    );
                    apply_world_updates(&mut framed_updates, updates);
                }
                NetEvent::Message(ServerMessagePayload::Ping) => {
                    if !multiplayer_room_state.has_sent_join_package {
                        let connection = (&mut connections)
                            .join()
                            .next()
                            .expect("Expected a server connection");

                        multiplayer_room_state.has_sent_join_package = true;
                        send_message_reliable(
                            connection,
                            &ClientMessagePayload::JoinRoom {
                                nickname: multiplayer_room_state.nickname.clone(),
                            },
                        );
                    }
                }
                // TODO: handle disconnects.
                _ => {}
            }
        }
    }
}

fn apply_world_updates(
    framed_updates: &mut FramedUpdates<ServerWorldUpdate>,
    incoming_updates: Vec<ServerWorldUpdate>,
) {
    let start_frame_number = incoming_updates.first().map(|update| update.frame_number);
    if let Some(start_frame_number) = start_frame_number {
        framed_updates.oldest_updated_frame = start_frame_number;
        let mut server_updates_iter = incoming_updates.into_iter();
        let mut frame_number = start_frame_number;

        // There may be updates altering the old ones, merge them if such exist.
        for frame_updates in framed_updates.updates_iter_mut(start_frame_number) {
            let server_update = server_updates_iter.next();
            if server_update.is_none() {
                return;
            }
            let server_update = server_update.unwrap();
            frame_number = server_update.frame_number;
            frame_updates.merge_another_update(server_update);
        }

        // Add all the other updates.
        if let Some(newest_update) = framed_updates.updates.back() {
            assert_eq!(newest_update.frame_number, frame_number);
        }
        framed_updates
            .updates
            .append(&mut VecDeque::from_iter(server_updates_iter));
    }
}
