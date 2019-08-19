#[cfg(not(feature = "client"))]
use amethyst::ecs::{Join, WriteStorage};
use amethyst::network::{NetEvent, NetPacket};

#[cfg(feature = "client")]
use ha_core::net::client_message::ClientMessage;
#[cfg(not(feature = "client"))]
use ha_core::net::server_message::ServerMessage;
use ha_core::net::NetConnection;

#[cfg(not(feature = "client"))]
pub fn broadcast_message_reliable(
    net_connections: &mut WriteStorage<NetConnection>,
    message: &ServerMessage,
) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a broadcasted message"),
    ));
    for connection in net_connections.join() {
        connection.queue(send_message.clone());
    }
}

#[cfg(feature = "client")]
pub fn send_message_reliable(net_connection: &mut NetConnection, message: &ClientMessage) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a client message"),
    ));
    net_connection.queue(send_message);
}

#[cfg(not(feature = "client"))]
pub fn send_message_reliable(net_connection: &mut NetConnection, message: &ServerMessage) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a server message"),
    ));
    net_connection.queue(send_message);
}
