use serde_derive::{Deserialize, Serialize};

use crate::ecs::resources::MultiplayerRoomPlayer;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    UpdateRoomPlayers(Vec<MultiplayerRoomPlayer>),
    Ping,
}

impl ServerMessage {
    pub fn is_ping_message(&self) -> bool {
        if let ServerMessage::Ping = self {
            true
        } else {
            false
        }
    }
}
