use serde_derive::{Deserialize, Serialize};

use crate::{ecs::resources::MultiplayerRoomPlayer, net::EntityNetIdentifier};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessagePayload {
    UpdateRoomPlayers(Vec<MultiplayerRoomPlayer>),
    /// Must have the same length as a last sent UpdateRoomPlayers,
    /// contains server ids for corresponding players.
    StartGame(Vec<EntityNetIdentifier>),
    Ping,
}

impl ServerMessagePayload {
    pub fn is_ping_message(&self) -> bool {
        if let ServerMessagePayload::Ping = self {
            true
        } else {
            false
        }
    }
}
