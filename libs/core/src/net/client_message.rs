use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    JoinRoom { nickname: String },
    Ping,
}

impl ClientMessage {
    pub fn is_ping_message(&self) -> bool {
        if let ClientMessage::Ping = self {
            true
        } else {
            false
        }
    }
}
