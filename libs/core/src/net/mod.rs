pub mod client_message;
pub mod server_message;

use amethyst::network;

pub type EncodedMessage = Vec<u8>;
pub type NetConnection = network::NetConnection<EncodedMessage>;
