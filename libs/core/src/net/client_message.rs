use serde_derive::{Deserialize, Serialize};

use crate::{
    actions::player::{PlayerCastAction, PlayerWalkAction},
    ecs::resources::world::{PlayerActionUpdate, PlayerLookActionUpdates},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessagePayload {
    JoinRoom { nickname: String },
    StartHostedGame,
    AcknowledgeWorldUpdate(u64),
    WalkAction(PlayerActionUpdate<PlayerWalkAction>),
    CastAction(PlayerActionUpdate<PlayerCastAction>),
    LookActions(PlayerLookActionUpdates),
    Ping,
}

impl ClientMessagePayload {
    pub fn is_ping_message(&self) -> bool {
        if let ClientMessagePayload::Ping = self {
            true
        } else {
            false
        }
    }
}
