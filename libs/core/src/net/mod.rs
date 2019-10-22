use serde_derive::{Deserialize, Serialize};

pub mod client_message;
pub mod server_message;

use amethyst::network::Bytes;

use crate::ecs::components::WorldPosition;

pub type EncodedMessage = Bytes;
pub type NetIdentifier = u64;

pub const INTERPOLATION_FRAME_DELAY: u64 = 10;

pub struct ConnectionNetEvent<T> {
    pub connection_id: NetIdentifier,
    pub event: NetEvent<T>,
}

pub enum NetEvent<T> {
    Connected,
    Message(T),
    Disconnected,
}

pub trait NetIdentifiable {
    fn net_id(&self) -> NetIdentifier;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetUpdate<T> {
    pub entity_net_id: NetIdentifier,
    pub data: T,
}

impl<T> NetIdentifiable for NetUpdate<T> {
    fn net_id(&self) -> NetIdentifier {
        self.entity_net_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetUpdateWithPosition<T> {
    pub entity_net_id: NetIdentifier,
    pub position: WorldPosition,
    pub data: T,
}

impl<T> NetIdentifiable for NetUpdateWithPosition<T> {
    fn net_id(&self) -> NetIdentifier {
        self.entity_net_id
    }
}
