use amethyst::{
    ecs::{Entities, Join, System, WriteExpect, WriteStorage},
    network::{NetEvent as AmethystNetEvent, NetPacket as AmethystNetPacket},
};
use log;

use std::time::{Duration, Instant};

use ha_core::{
    ecs::{components::NetConnectionModel, system_data::time::GameTimeService},
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload, EncodedMessage,
        NetConnection, NetIdentifier,
    },
};

use crate::ecs::resources::ConnectionEvents;
use ha_core::net::{ConnectionNetEvent, NetEvent};

const PING_INTERVAL_MILLIS: u64 = 500;

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
    connection_id_autoinc: NetIdentifier,
    ping_id_autoinc: NetIdentifier,
}

impl NetConnectionManagerSystem {
    fn next_connection_id(&mut self) -> NetIdentifier {
        let id = self.connection_id_autoinc;
        self.connection_id_autoinc = self.connection_id_autoinc.wrapping_add(1);
        id
    }

    fn next_ping_id(&mut self) -> NetIdentifier {
        let id = self.ping_id_autoinc;
        self.ping_id_autoinc = self.ping_id_autoinc.wrapping_add(1);
        id
    }
}

impl<'s> System<'s> for NetConnectionManagerSystem {
    type SystemData = (
        GameTimeService<'s>,
        WriteExpect<'s, ConnectionEvents>,
        WriteStorage<'s, NetConnection>,
        WriteStorage<'s, NetConnectionModel>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            mut incoming_messages,
            mut connections,
            mut net_connection_models,
            entities,
        ): Self::SystemData,
    ) {
        let mut count = 0;
        let mut connection_count = 0;

        let ping_id = self.next_ping_id();
        let send_ping_packet =
            AmethystNetEvent::Packet(AmethystNetPacket::unreliable(ping_message(ping_id)));

        for (e, connection) in (&entities, &mut connections).join() {
            let mut connection_messages_count = 0;
            let mut connection_model = net_connection_models
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

            let mut responses = Vec::new();
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

                let (event, response) = process_connection_event(
                    &ev,
                    &mut connection_model,
                    game_time_service.game_frame_number(),
                );

                if let Some(event) = event {
                    connection_messages_count += 1;
                    incoming_messages.0.push(event);
                }
                if let Some(response) = response {
                    responses.push(response);
                }
            }
            for response in responses {
                connection.queue(response);
            }

            if connection_model.last_pinged_at + Duration::from_millis(PING_INTERVAL_MILLIS)
                < Instant::now()
            {
                connection_model.last_pinged_at = Instant::now();
                connection_model
                    .ping_pong_data
                    .add_ping(ping_id, game_time_service.game_frame_number());
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

fn ping_message(ping_id: NetIdentifier) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage::Ping(ping_id))
        .expect("Expected to serialize Ping message")
}

fn pong_message(ping_id: NetIdentifier, frame_number: u64) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage::Pong {
        ping_id,
        frame_number,
    })
    .expect("Expected to serialize Pong message")
}

fn process_connection_event(
    ev: &AmethystNetEvent<EncodedMessage>,
    connection_model: &mut NetConnectionModel,
    frame_number: u64,
) -> (
    Option<ConnectionNetEvent<IncomingMessage>>,
    Option<AmethystNetEvent<EncodedMessage>>,
) {
    let connection_id = connection_model.id;
    match ev {
        AmethystNetEvent::Packet(packet) => {
            if let Ok(message) = bincode::deserialize::<IncomingMessage>(packet.content()) {
                match message {
                    IncomingMessage::Ping(ping_id) => {
                        log::trace!("Received a new ping message: {:?}", &message);
                        (
                            None,
                            Some(AmethystNetEvent::Packet(AmethystNetPacket::unreliable(
                                pong_message(ping_id, frame_number),
                            ))),
                        )
                    }
                    IncomingMessage::Pong {
                        ping_id,
                        frame_number: peer_frame_number,
                    } => {
                        log::trace!("Received a new pong message: {:?}", &message);
                        connection_model.ping_pong_data.add_pong(
                            ping_id,
                            peer_frame_number,
                            frame_number,
                        );
                        (None, None)
                    }
                    _ => {
                        log::debug!("Received a new message: {:?}", &message);
                        (
                            Some(ConnectionNetEvent {
                                connection_id,
                                event: NetEvent::Message(message),
                            }),
                            None,
                        )
                    }
                }
            } else {
                (None, None)
            }
        }
        AmethystNetEvent::Connected(_addr) => (
            Some(ConnectionNetEvent {
                connection_id,
                event: NetEvent::Connected,
            }),
            None,
        ),
        AmethystNetEvent::Disconnected(_addr) => (
            Some(ConnectionNetEvent {
                connection_id,
                event: NetEvent::Disconnected,
            }),
            None,
        ),
        _ => (None, None),
    }
}
