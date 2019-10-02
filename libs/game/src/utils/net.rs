#[cfg(not(feature = "client"))]
use amethyst::ecs::{Join, WriteStorage};
use amethyst::network::{NetEvent, NetPacket};

#[cfg(feature = "client")]
use ha_core::net::client_message::ClientMessagePayload;
#[cfg(not(feature = "client"))]
use ha_core::net::server_message::ServerMessagePayload;
use ha_core::net::NetConnection;

#[cfg(not(feature = "client"))]
pub fn broadcast_message_reliable(
    net_connections: &mut WriteStorage<NetConnection>,
    message: &ServerMessagePayload,
) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a broadcasted message"),
    ));
    for connection in net_connections.join() {
        connection.queue(send_message.clone());
    }
}

#[cfg(feature = "client")]
pub fn send_message_reliable(net_connection: &mut NetConnection, message: &ClientMessagePayload) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a client message"),
    ));
    net_connection.queue(send_message);
}

#[cfg(not(feature = "client"))]
pub fn send_message_reliable(net_connection: &mut NetConnection, message: &ServerMessagePayload) {
    let send_message = NetEvent::Packet(NetPacket::reliable_unordered(
        bincode::serialize(&message).expect("Expected to serialize a server message"),
    ));
    net_connection.queue(send_message);
}

#[cfg(feature = "client")]
pub fn send_message_unreliable(net_connection: &mut NetConnection, message: &ClientMessagePayload) {
    log::trace!("Sending: {:#?}", message);
    let send_message = NetEvent::Packet(NetPacket::unreliable(
        bincode::serialize(&message).expect("Expected to serialize a client message"),
    ));
    net_connection.queue(send_message);
}

#[cfg(not(feature = "client"))]
pub fn send_message_unreliable(net_connection: &mut NetConnection, message: &ServerMessagePayload) {
    log::trace!("Sending: {:#?}", message);
    let packet = NetPacket::unreliable(
        bincode::serialize(&message).expect("Expected to serialize a server message"),
    );
    log::trace!("Packet len: {}", packet.content().len());
    let send_message = NetEvent::Packet(packet);
    net_connection.queue(send_message);
}
