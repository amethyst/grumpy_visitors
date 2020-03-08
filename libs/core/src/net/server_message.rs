use serde_derive::{Deserialize, Serialize};

use crate::{
    ecs::resources::{net::MultiplayerRoomPlayer, world::ServerWorldUpdate},
    net::NetIdentifier,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessagePayload {
    Heartbeat,
    UpdateRoomPlayers(Vec<MultiplayerRoomPlayer>),
    /// Must have the same length as a last sent UpdateRoomPlayers,
    /// contains server ids for corresponding players.
    StartGame(Vec<NetIdentifier>),
    Handshake {
        net_id: NetIdentifier,
        is_host: bool,
    },
    UpdateWorld {
        id: u64,
        updates: Vec<ServerWorldUpdate>,
    },
    DiscardWalkActions(Vec<NetIdentifier>),
    Ping(NetIdentifier),
    Pong {
        ping_id: NetIdentifier,
        frame_number: u64,
    },
    /// Contains connection ids of players a server is waiting for.
    PauseWaitingForPlayers {
        id: NetIdentifier,
        players: Vec<NetIdentifier>,
    },
    UnpauseWaitingForPlayers(NetIdentifier),
    Disconnect(DisconnectReason),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DisconnectReason {
    GameIsAlreadyStarted,
    Kick,
}

impl ServerMessagePayload {
    pub fn is_heartbeat(&self) -> bool {
        if let Self::Heartbeat = *self {
            true
        } else {
            false
        }
    }
}
