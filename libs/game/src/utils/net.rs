use amethyst::network::simulation::{DeliveryRequirement, TransportResource, UrgencyRequirement};

use gv_core::ecs::components::NetConnectionModel;
#[cfg(feature = "client")]
use gv_core::net::client_message::ClientMessagePayload;
#[cfg(not(feature = "client"))]
use gv_core::net::server_message::ServerMessagePayload;

// TODO - jstnlef: I made all of the UrgencyRequirements immediate because I wasn't sure if you wanted
// to use the NetworkSimulationTime resource to handle rate of send.

#[cfg(not(feature = "client"))]
pub fn broadcast_message_reliable<'a>(
    transport: &mut TransportResource,
    net_connections: impl Iterator<Item = &'a NetConnectionModel>,
    message: &ServerMessagePayload,
) {
    let send_message =
        bincode::serialize(message).expect("Expected to serialize a broadcasted message");
    for connection in net_connections {
        transport.send_with_requirements(
            connection.addr,
            &send_message,
            DeliveryRequirement::Reliable,
            UrgencyRequirement::Immediate,
        );
    }
}

#[cfg(feature = "client")]
pub fn send_message_reliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    message: &ClientMessagePayload,
) {
    let send_message = bincode::serialize(message).expect("Expected to serialize a client message");
    transport.send_with_requirements(
        net_connection.addr,
        &send_message,
        DeliveryRequirement::Reliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(not(feature = "client"))]
pub fn send_message_reliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    message: &ServerMessagePayload,
) {
    let send_message = bincode::serialize(message).expect("Expected to serialize a server message");
    transport.send_with_requirements(
        net_connection.addr,
        &send_message,
        DeliveryRequirement::Reliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(feature = "client")]
pub fn send_message_unreliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    message: &ClientMessagePayload,
) {
    log::trace!("Sending: {:#?}", message);
    let send_message = bincode::serialize(message).expect("Expected to serialize a client message");
    transport.send_with_requirements(
        net_connection.addr,
        &send_message,
        DeliveryRequirement::Unreliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(not(feature = "client"))]
pub fn send_message_unreliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    message: &ServerMessagePayload,
) {
    log::trace!("Sending: {:#?}", message);
    let send_message = bincode::serialize(message).expect("Expected to serialize a server message");
    log::trace!("Packet len: {}", send_message.len());
    transport.send_with_requirements(
        net_connection.addr,
        &send_message,
        DeliveryRequirement::Unreliable,
        UrgencyRequirement::Immediate,
    );
}
