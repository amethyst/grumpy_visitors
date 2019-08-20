pub mod client_message;
pub mod server_message;

use amethyst::network;

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
