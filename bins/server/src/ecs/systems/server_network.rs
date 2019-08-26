use amethyst::ecs::{System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::resources::{
        net::{MultiplayerGameState, MultiplayerRoomPlayer},
        GameEngineState, NewGameEngineState,
    },
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload,
        ConnectionIdentifier, NetConnection, NetEvent,
    },
};
use ha_game::{ecs::resources::ConnectionEvents, utils::net::broadcast_message_reliable};

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
        WriteExpect<'s, ConnectionEvents>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, NewGameEngineState>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            mut connection_events,
            mut multiplayer_game_state,
            mut new_game_engine_state,
            mut net_connections,
        ): Self::SystemData,
    ) {
        for connection_event in connection_events.0.drain(..) {
            let connection_id = connection_event.connection_id;
            match connection_event.event {
                NetEvent::Message(ClientMessagePayload::JoinRoom { nickname }) => {
                    // TODO: we'll need a more reliable way to determine the host in future.
                    let is_host = if multiplayer_game_state.players.is_empty() {
                        self.host_connection_id = connection_id;
                        true
                    } else {
                        self.host_connection_id == connection_id
                    };
                    multiplayer_game_state
                        .update_players()
                        .push(MultiplayerRoomPlayer {
                            connection_id,
                            entity_net_id: 0,
                            nickname,
                            is_host,
                        });
                }
                NetEvent::Message(ClientMessagePayload::StartHostedGame)
                    if connection_id == self.host_connection_id =>
                {
                    multiplayer_game_state.is_playing = true;
                    new_game_engine_state.0 = GameEngineState::Playing;
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
    }
}
