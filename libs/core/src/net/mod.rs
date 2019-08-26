use serde_derive::{Deserialize, Serialize};

pub mod client_message;
pub mod server_message;

use amethyst::network;

use crate::ecs::components::WorldPosition;

pub type EncodedMessage = Vec<u8>;
pub type NetConnection = network::NetConnection<EncodedMessage>;
pub type EntityNetIdentifier = u64;
pub type ConnectionIdentifier = usize;

pub struct ConnectionNetEvent<T> {
    pub connection_id: usize,
    pub event: NetEvent<T>,
}

pub enum NetEvent<T> {
    Connected,
    Message(T),
    Disconnected,
}

pub trait IdentifiableNetUpdate {
    fn entity_net_identifier(&self) -> EntityNetIdentifier;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetUpdate<T> {
    pub entity_net_identifier: EntityNetIdentifier,
    pub data: T,
}

impl<T> IdentifiableNetUpdate for NetUpdate<T> {
    fn entity_net_identifier(&self) -> EntityNetIdentifier {
        self.entity_net_identifier
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetUpdateWithPosition<T> {
    pub entity_net_identifier: EntityNetIdentifier,
    pub position: WorldPosition,
    pub data: T,
}

impl<T> IdentifiableNetUpdate for NetUpdateWithPosition<T> {
    fn entity_net_identifier(&self) -> EntityNetIdentifier {
        self.entity_net_identifier
    }
}

pub trait MergableNetUpdates {
    fn merge(&mut self, other: Self);
}

impl<T: IdentifiableNetUpdate> MergableNetUpdates for Vec<T> {
    fn merge(&mut self, mut other: Self) {
        self.retain(|update| {
            !other
                .iter()
                .any(|other| update.entity_net_identifier() == other.entity_net_identifier())
        });
        self.append(&mut other);
    }
}
