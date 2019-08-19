use amethyst::ecs::{System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::resources::{MultiplayerRoomPlayer, MultiplayerRoomPlayers},
    net::{client_message::ClientMessage, server_message::ServerMessage, NetConnection},
};
use ha_game::{ecs::resources::IncomingMessages, utils::net::broadcast_message_reliable};

pub struct ServerNetworkSystem;

impl<'s> System<'s> for ServerNetworkSystem {
    type SystemData = (
        WriteExpect<'s, IncomingMessages>,
        WriteExpect<'s, MultiplayerRoomPlayers>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (mut incoming_messages, mut multiplayer_room_players, mut net_connections): Self::SystemData,
    ) {
        // TODO: process disconnects.
        for message in incoming_messages.0.drain(..) {
            #[allow(clippy::single_match)]
            match message {
                ClientMessage::JoinRoom { nickname } => {
                    // TODO: we'll need a better way to determine the host in future.
                    let is_host = multiplayer_room_players.players.is_empty();
                    multiplayer_room_players
                        .update()
                        .push(MultiplayerRoomPlayer { nickname, is_host });
                }
                _ => {}
            }
        }

        if let Some(players) = multiplayer_room_players.read_updated() {
            broadcast_message_reliable(
                &mut net_connections,
                &ServerMessage::UpdateRoomPlayers(players.to_owned()),
            );
        }
    }
}
