use amethyst::ecs::{Entities, Join, System, WriteExpect, WriteStorage};

use ha_client_shared::ecs::resources::MultiplayerRoomState;
use ha_core::{
    ecs::resources::{GameEngineState, MultiplayerGameState, NewGameEngineState},
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
                NetEvent::Message(ServerMessagePayload::Ping) => {
                    let connection = (&mut connections)
                        .join()
                        .next()
                        .expect("Expected a server connection");

                    if !multiplayer_room_state.has_sent_join_package {
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
