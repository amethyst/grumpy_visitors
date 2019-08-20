use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessagePayload {
    JoinRoom { nickname: String },
    StartHostedGame,
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
