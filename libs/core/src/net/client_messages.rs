use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMessages {
    JoinRoom { nickname: String },
    Ping,
}
