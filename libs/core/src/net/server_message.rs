use serde_derive::{Deserialize, Serialize};

use crate::{
    ecs::resources::{net::MultiplayerRoomPlayer, world::ServerWorldUpdate},
    net::NetIdentifier,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub session_id: NetIdentifier,
    pub payload: ServerMessagePayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessagePayload {
    Heartbeat,
    UpdateRoomPlayers(Vec<MultiplayerRoomPlayer>),
    /// Must have the same length as a last sent UpdateRoomPlayers,
    /// contains server (entity) ids for corresponding players.
    StartGame(Vec<(NetIdentifier, MultiplayerRoomPlayer)>),
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
    ReportPlayersNetStatus {
        id: NetIdentifier,
        players: Vec<PlayerNetStatus>,
    },
    /// Contains connection ids of players a server is waiting for.
    PauseWaitingForPlayers {
        id: NetIdentifier,
        players: Vec<NetIdentifier>,
    },
    UnpauseWaitingForPlayers(NetIdentifier),
    Disconnect(DisconnectReason),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerNetStatus {
    pub connection_id: NetIdentifier,
    pub frame_number: u64,
    pub average_lagging_behind: u64,
    pub latency_ms: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// For rejecting any connections while the server
    /// isn't connected to a host (in case of self-hosting).
    Uninitialized,
    GameIsStarted,
    RoomIsFull,
    Kick,
    Closed,
    ServerCrashed(i32),
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
