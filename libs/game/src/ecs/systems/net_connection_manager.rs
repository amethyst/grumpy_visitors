use amethyst::{
    core::SystemDesc,
    ecs::{Entities, Read, ReaderId, System, SystemData, World, WriteExpect, WriteStorage, Write},
    network::simulation::{NetworkSimulationEvent, TransportResource},
    shrev::EventChannel
};
use log;

use std::time::{Duration, Instant};

use gv_core::{
    ecs::{components::NetConnectionModel, system_data::time::GameTimeService},
    net::{
        client_message::ClientMessagePayload, server_message::ServerMessagePayload,
        ConnectionNetEvent, EncodedMessage, NetEvent, NetIdentifier,
    },
};

use crate::ecs::resources::ConnectionEvents;

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
pub struct NetConnectionManagerDesc;

impl<'a, 'b> SystemDesc<'a, 'b, NetConnectionManagerSystem> for NetConnectionManagerDesc {
    fn build(self, world: &mut World) -> NetConnectionManagerSystem {
        <NetConnectionManagerSystem as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        NetConnectionManagerSystem::new(reader)
    }
}

pub struct NetConnectionManagerSystem {
    connection_id_autoinc: NetIdentifier,
    ping_id_autoinc: NetIdentifier,
    reader: ReaderId<NetworkSimulationEvent>
}

impl NetConnectionManagerSystem {
    fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self {
            connection_id_autoinc: Default::default(),
            ping_id_autoinc: Default::default(),
            reader
        }
    }

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
        Write<'s, TransportResource>,
        Read<'s, EventChannel<NetworkSimulationEvent>>,
        WriteExpect<'s, ConnectionEvents>,
        WriteStorage<'s, NetConnectionModel>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            mut transport,
            net_events,
            mut incoming_messages,
            mut net_connection_models,
            entities,
        ): Self::SystemData,
    ) {
        let mut count = 0;
        let mut connection_count = 0;

        let ping_id = self.next_ping_id();
        let send_ping_packet = ping_message(ping_id);

//        let mut responses = Vec::new();

        for event in net_events.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Connect(addr) => {
//                    log::info!("New connection ({}): {}", connection_model.id, addr)
                    // TODO: Find this connection by ID else insert it.
                }
                NetworkSimulationEvent::Disconnect(_addr) => {
                    // TODO: Delete from the list of entities.
//                    log::info!(
//                            "Dropping a connection ({}) to {}...",
//                            connection_model.id,
//                            connection.target_addr
//                        );
//                    entities
//                        .delete(e)
//                        .expect("Expected to delete a ConnectionReader");
                }
                _ => {}
            }

//            let (event, response) =
//                process_connection_event(&event, &mut connection_model, &game_time_service);
//
//            if let Some(event) = event {
//                connection_messages_count += 1;
//                incoming_messages.0.push(event);
//            }
//            if let Some(response) = response {
//                responses.push(response);
//            }
        }


//        for (e, connection) in (&entities, &mut connections).join() {
//            let mut connection_messages_count = 0;
//            let mut connection_model = net_connection_models
//                .entry(e)
//                .expect("Expected to get the right generation")
//                .or_insert_with(|| {
//                    let connection_id = self.next_connection_id();
//                    incoming_messages.0.push(ConnectionNetEvent {
//                        connection_id,
//                        event: NetEvent::Connected,
//                    });
//                    NetConnectionModel::new(connection_id, connection.register_reader())
//                });
//
//            let mut responses = Vec::new();
//
//            for ev in connection.received_events(&mut self.reader) {
//                match ev {
//                    NetworkSimulationEvent::Connect(addr) => {
//                        log::info!("New connection ({}): {}", connection_model.id, addr)
//                    }
//                    NetworkSimulationEvent::Disconnect(_addr) => {
//                        log::info!(
//                            "Dropping a connection ({}) to {}...",
//                            connection_model.id,
//                            connection.target_addr
//                        );
//                        entities
//                            .delete(e)
//                            .expect("Expected to delete a ConnectionReader");
//                    }
//                    _ => {}
//                }
//
//                let (event, response) =
//                    process_connection_event(&ev, &mut connection_model, &game_time_service);
//
//                if let Some(event) = event {
//                    connection_messages_count += 1;
//                    incoming_messages.0.push(event);
//                }
//                if let Some(response) = response {
//                    responses.push(response);
//                }
//            }
//            for response in responses {
//                connection.queue(response);
//            }
//
//            if connection_model.ping_pong_data.last_pinged_at
//                + Duration::from_millis(PING_INTERVAL_MILLIS)
//                < Instant::now()
//            {
//                connection_model
//                    .ping_pong_data
//                    .add_ping(ping_id, game_time_service.engine_time().frame_number());
//                connection.queue(send_ping_packet.clone());
//            }
//
//            if connection_messages_count > 0 {
//                connection_count += 1;
//                count += connection_messages_count;
//            }
//        }

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
        .expect("Expected to serialize Ping message").into()
}

fn pong_message(ping_id: NetIdentifier, frame_number: u64) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage::Pong {
        ping_id,
        frame_number,
    })
    .expect("Expected to serialize Pong message").into()
}

fn process_connection_event(
    ev: &NetworkSimulationEvent,
    connection_model: &mut NetConnectionModel,
    game_time_service: &GameTimeService,
) -> (
    Option<ConnectionNetEvent<IncomingMessage>>,
    Option<EncodedMessage>,
) {
    let connection_id = connection_model.id;
    match ev {
        NetworkSimulationEvent::Message(addr, bytes) => {
            if let Ok(message) = bincode::deserialize::<IncomingMessage>(bytes.as_ref()) {
                match message {
                    IncomingMessage::Ping(ping_id) => {
                        log::trace!("Received a new ping message: {:?}", &message);
                        (
                            None,
                            Some(pong_message(ping_id, game_time_service.game_frame_number())),
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
                            game_time_service.engine_time().frame_number(),
                            game_time_service.game_frame_number(),
                        );
                        (None, None)
                    }
                    message if message.is_heartbeat() => {
                        log::trace!("Received a new Heartbeat message");
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
        NetworkSimulationEvent::Connect(_addr) => (
            Some(ConnectionNetEvent {
                connection_id,
                event: NetEvent::Connected,
            }),
            None,
        ),
        NetworkSimulationEvent::Disconnect(_addr) => (
            Some(ConnectionNetEvent {
                connection_id,
                event: NetEvent::Disconnected,
            }),
            None,
        ),
        _ => (None, None),
    }
}
