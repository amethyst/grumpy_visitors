use amethyst::network::simulation::{DeliveryRequirement, TransportResource, UrgencyRequirement};

use gv_core::ecs::components::NetConnectionModel;
#[cfg(feature = "client")]
use gv_core::net::client_message::{ClientMessage, ClientMessagePayload};
#[cfg(not(feature = "client"))]
use gv_core::net::server_message::{ServerMessage, ServerMessagePayload};

#[cfg(not(feature = "client"))]
pub fn broadcast_message_reliable<'a>(
    transport: &mut TransportResource,
    net_connections: impl Iterator<Item = &'a NetConnectionModel>,
    payload: ServerMessagePayload,
) {
    for connection in net_connections {
        let sent_message = bincode::serialize(&ServerMessage {
            session_id: connection.session_id,
            payload: payload.clone(),
        })
        .expect("Expected to serialize a broadcasted message");
        if !connection.disconnected {
            transport.send_with_requirements(
                connection.addr,
                &sent_message,
                DeliveryRequirement::Reliable,
                UrgencyRequirement::Immediate,
            );
        }
    }
}

#[cfg(feature = "client")]
pub fn send_message_reliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    payload: ClientMessagePayload,
) {
    if net_connection.disconnected {
        return;
    }
    let sent_message = bincode::serialize(&ClientMessage {
        session_id: net_connection.session_id,
        payload,
    })
    .expect("Expected to serialize a client message");
    transport.send_with_requirements(
        net_connection.addr,
        &sent_message,
        DeliveryRequirement::Reliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(not(feature = "client"))]
pub fn send_message_reliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    payload: ServerMessagePayload,
) {
    if net_connection.disconnected {
        return;
    }
    let sent_message = bincode::serialize(&ServerMessage {
        session_id: net_connection.session_id,
        payload,
    })
    .expect("Expected to serialize a server message");
    transport.send_with_requirements(
        net_connection.addr,
        &sent_message,
        DeliveryRequirement::Reliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(feature = "client")]
pub fn send_message_unreliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    payload: ClientMessagePayload,
) {
    if net_connection.disconnected {
        return;
    }
    let message = ClientMessage {
        session_id: net_connection.session_id,
        payload,
    };
    log::trace!("Sending: {:#?}", message);
    let sent_message =
        bincode::serialize(&message).expect("Expected to serialize a client message");
    transport.send_with_requirements(
        net_connection.addr,
        &sent_message,
        DeliveryRequirement::Unreliable,
        UrgencyRequirement::Immediate,
    );
}

#[cfg(not(feature = "client"))]
pub fn send_message_unreliable(
    transport: &mut TransportResource,
    net_connection: &NetConnectionModel,
    payload: ServerMessagePayload,
) {
    if net_connection.disconnected {
        return;
    }
    let message = ServerMessage {
        session_id: net_connection.session_id,
        payload,
    };
    log::trace!("Sending: {:#?}", message);
    let sent_message =
        bincode::serialize(&message).expect("Expected to serialize a server message");
    log::trace!("Packet len: {}", sent_message.len());
    transport.send_with_requirements(
        net_connection.addr,
        &sent_message,
        DeliveryRequirement::Unreliable,
        UrgencyRequirement::Immediate,
    );
}
