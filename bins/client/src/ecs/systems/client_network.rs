use amethyst::ecs::{Join, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::resources::MultiplayerRoomPlayers,
    net::{client_message::ClientMessage, server_message::ServerMessage, NetConnection},
};
use ha_game::{ecs::resources::IncomingMessages, utils::net::send_message_reliable};

use crate::ecs::resources::MultiplayerRoomState;

pub struct ClientNetworkSystem;

impl<'s> System<'s> for ClientNetworkSystem {
    type SystemData = (
        WriteExpect<'s, IncomingMessages>,
        WriteExpect<'s, MultiplayerRoomState>,
        WriteExpect<'s, MultiplayerRoomPlayers>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            mut incoming_messages,
            mut multiplayer_room_state,
            mut multiplayer_room_players,
            mut connections,
        ): Self::SystemData,
    ) {
        for message in incoming_messages.0.drain(..) {
            match message {
                ServerMessage::UpdateRoomPlayers(players) => {
                    log::info!("Updated room players");
                    *multiplayer_room_players.update() = players;
                }
                ServerMessage::Ping => {
                    let connection = (&mut connections)
                        .join()
                        .next()
                        .expect("Expected a server connection");

                    if !multiplayer_room_state.has_sent_join_package {
                        multiplayer_room_state.has_sent_join_package = true;
                        send_message_reliable(
                            connection,
                            &ClientMessage::JoinRoom {
                                nickname: multiplayer_room_state.nickname.clone(),
                            },
                        );
                    }
                }
            }
        }
    }
}
