use serde_derive::{Deserialize, Serialize};

use crate::{
    actions::player::{PlayerCastAction, PlayerWalkAction},
    ecs::resources::world::{ImmediatePlayerActionsUpdates, PlayerLookActionUpdates},
    net::NetIdentifier,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessagePayload {
    JoinRoom {
        nickname: String,
    },
    StartHostedGame,
    AcknowledgeWorldUpdate(u64),
    WalkActions(ImmediatePlayerActionsUpdates<PlayerWalkAction>),
    CastActions(ImmediatePlayerActionsUpdates<PlayerCastAction>),
    LookActions(PlayerLookActionUpdates),
    Ping(NetIdentifier),
    Pong {
        ping_id: NetIdentifier,
        frame_number: u64,
    },
}

impl ClientMessagePayload {
    pub fn is_heartbeat(&self) -> bool {
        false
    }
}
