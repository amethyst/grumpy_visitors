pub mod client_messages;
pub mod server_messages;

use amethyst::network;

pub type EncodedMessage = Vec<u8>;
pub type NetConnection = network::NetConnection<EncodedMessage>;
