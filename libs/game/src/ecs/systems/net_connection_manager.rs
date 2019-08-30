use amethyst::{
    ecs::{Entities, Join, System, WriteExpect, WriteStorage},
    network::{NetEvent as AmethystNetEvent, NetPacket as AmethystNetPacket},
};
use log;

use std::time::{Duration, Instant};

use ha_core::{
    ecs::components::NetConnectionModel,
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload,
        ConnectionIdentifier, EncodedMessage, NetConnection,
    },
};

use crate::ecs::resources::ConnectionEvents;
use ha_core::net::{ConnectionNetEvent, NetEvent};

const PING_INTERVAL_SECS: u64 = 1;

#[cfg(feature = "client")]
type IncomingMessage = ServerMessagePayload;
#[cfg(not(feature = "client"))]
type IncomingMessage = ClientMessagePayload;
#[cfg(feature = "client")]
type OutcomingMessage = ClientMessagePayload;
#[cfg(not(feature = "client"))]
type OutcomingMessage = ServerMessagePayload;

#[derive(Default)]
pub struct NetConnectionManagerSystem {
    connection_id_autoinc: ConnectionIdentifier,
}

impl NetConnectionManagerSystem {
    pub fn new() -> Self {
        Self {
            connection_id_autoinc: 0,
        }
    }

    fn next_connection_id(&mut self) -> ConnectionIdentifier {
        let id = self.connection_id_autoinc;
        self.connection_id_autoinc = self.connection_id_autoinc.wrapping_add(1);
        id
    }
}

impl<'s> System<'s> for NetConnectionManagerSystem {
    type SystemData = (
        WriteExpect<'s, ConnectionEvents>,
        WriteStorage<'s, NetConnection>,
        WriteStorage<'s, NetConnectionModel>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (mut incoming_messages, mut connections, mut net_connection_models, entities): Self::SystemData,
    ) {
        let mut count = 0;
        let mut connection_count = 0;

        let send_ping_packet =
            AmethystNetEvent::Packet(AmethystNetPacket::unreliable(ping_message()));

        for (e, connection) in (&entities, &mut connections).join() {
            let mut connection_messages_count = 0;
            let connection_model = net_connection_models
                .entry(e)
                .expect("Expected to get the right generation")
                .or_insert_with(|| {
                    let connection_id = self.next_connection_id();
                    incoming_messages.0.push(ConnectionNetEvent {
                        connection_id,
                        event: NetEvent::Connected,
                    });
                    NetConnectionModel::new(connection_id, connection.register_reader())
                });

            for ev in connection.received_events(&mut connection_model.reader) {
                match ev {
                    AmethystNetEvent::Connected(addr) => {
                        log::info!("New connection ({}): {}", connection_model.id, addr)
                    }
                    AmethystNetEvent::Disconnected(_addr) => {
                        log::info!(
                            "Dropping a connection ({}) to {}...",
                            connection_model.id,
                            connection.target_addr
                        );
                        entities
                            .delete(e)
                            .expect("Expected to delete a ConnectionReader");
                    }
                    _ => {}
                }

                if let Some(event) = process_connection_event(&connection_model, &ev) {
                    if let ConnectionNetEvent {
                        event: NetEvent::Message(IncomingMessage::Ping),
                        ..
                    } = &event
                    {
                    } else {
                        connection_messages_count += 1;
                    }
                    incoming_messages.0.push(event);
                }
            }

            if connection_model.last_pinged_at + Duration::from_secs(PING_INTERVAL_SECS)
                < Instant::now()
            {
                connection_model.last_pinged_at = Instant::now();
                connection.queue(send_ping_packet.clone());
            }

            if connection_messages_count > 0 {
                connection_count += 1;
                count += connection_messages_count;
            }
        }

        if count > 0 {
            log::trace!(
                "Received {} messages this frame from {} connections",
                count,
                connection_count
            );
        }
    }
}

fn ping_message() -> Vec<u8> {
    bincode::serialize(&OutcomingMessage::Ping).expect("Expected to serialize Ping message")
}

fn process_connection_event(
    connection_model: &NetConnectionModel,
    ev: &AmethystNetEvent<EncodedMessage>,
) -> Option<ConnectionNetEvent<IncomingMessage>> {
    let connection_id = connection_model.id;
    match ev {
        AmethystNetEvent::Packet(packet) => {
            if let Ok(message) = bincode::deserialize::<IncomingMessage>(packet.content()) {
                if !message.is_ping_message() {
                    log::debug!("{:?}", &message);
                }
                Some(ConnectionNetEvent {
                    connection_id,
                    event: NetEvent::Message(message),
                })
            } else {
                None
            }
        }
        AmethystNetEvent::Connected(_addr) => Some(ConnectionNetEvent {
            connection_id,
            event: NetEvent::Connected,
        }),
        AmethystNetEvent::Disconnected(_addr) => Some(ConnectionNetEvent {
            connection_id,
            event: NetEvent::Disconnected,
        }),
        _ => None,
    }
}
