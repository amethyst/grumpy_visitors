use serde_derive::{Deserialize, Serialize};

use std::time::Duration;

use crate::{
    actions::{
        player::{PlayerCastAction, PlayerWalkAction},
        ClientActionUpdate,
    },
    ecs::resources::world::{ImmediatePlayerActionsUpdates, PlayerLookActionUpdates},
    net::NetIdentifier,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    pub session_id: NetIdentifier,
    pub payload: ClientMessagePayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessagePayload {
    Heartbeat,
    JoinRoom {
        // As the server stores session id coming with this message
        // (see `ServerMessage::session_id`), `sent_at` is used to filter out outdated handshakes
        // in case there are duplicates of reliable messages.
        sent_at: Duration,
        nickname: String,
    },
    StartHostedGame,
    AcknowledgeWorldUpdate(u64),
    WalkActions(ImmediatePlayerActionsUpdates<ClientActionUpdate<PlayerWalkAction>>),
    CastActions(ImmediatePlayerActionsUpdates<ClientActionUpdate<PlayerCastAction>>),
    LookActions(PlayerLookActionUpdates),
    Ping(NetIdentifier),
    Pong {
        ping_id: NetIdentifier,
        frame_number: u64,
    },
    Kick {
        /// Connection id stored by the host process.
        kicked_connection_id: NetIdentifier,
    },
    Disconnect,
}

impl ClientMessagePayload {
    pub fn is_heartbeat(&self) -> bool {
        if let Self::Heartbeat = *self {
            true
        } else {
            false
        }
    }
}
