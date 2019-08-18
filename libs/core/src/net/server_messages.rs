use serde_derive::{Deserialize, Serialize};

use crate::ecs::resources::MultiplayerRoomPlayers;

#[derive(Serialize, Deserialize)]
pub enum ServerMessages {
    UpdateRoomPlayers(MultiplayerRoomPlayers),
    Ping,
}
