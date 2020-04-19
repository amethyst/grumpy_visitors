use amethyst::{
    core::SystemDesc,
    ecs::{
        Entities, Join, Read, ReaderId, System, SystemData, World, Write, WriteExpect, WriteStorage,
    },
    network::simulation::{
        DeliveryRequirement, NetworkSimulationEvent, TransportResource, UrgencyRequirement,
    },
    shrev::EventChannel,
};

use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use gv_core::{
    ecs::{components::NetConnectionModel, system_data::time::GameTimeService},
    net::{
        client_message::{ClientMessage, ClientMessagePayload},
        server_message::{ServerMessage, ServerMessagePayload},
        ConnectionNetEvent, EncodedMessage, NetEvent, NetIdentifier,
    },
};

use crate::ecs::resources::ConnectionEvents;

const PING_INTERVAL_MILLIS: u64 = 500;

#[cfg(feature = "client")]
type IncomingMessage = ServerMessage;
#[cfg(not(feature = "client"))]
type IncomingMessage = ClientMessage;
#[cfg(feature = "client")]
type OutcomingMessage = ClientMessage;
#[cfg(not(feature = "client"))]
type OutcomingMessage = ServerMessage;

#[cfg(feature = "client")]
type IncomingMessagePayload = ServerMessagePayload;
#[cfg(not(feature = "client"))]
type IncomingMessagePayload = ClientMessagePayload;
#[cfg(feature = "client")]
type OutcomingMessagePayload = ClientMessagePayload;
#[cfg(not(feature = "client"))]
type OutcomingMessagePayload = ServerMessagePayload;

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
    reader: ReaderId<NetworkSimulationEvent>,
}

impl NetConnectionManagerSystem {
    fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self {
            connection_id_autoinc: Default::default(),
            ping_id_autoinc: Default::default(),
            reader,
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
            mut connection_events,
            mut net_connection_models,
            entities,
        ): Self::SystemData,
    ) {
        let ping_id = self.next_ping_id();

        // A hacky way to update connection_id_autoinc if some connections have been already created
        // (like we do it in ServerNetworkSystem).
        if !net_connection_models.is_empty() && self.connection_id_autoinc == 0 {
            self.connection_id_autoinc = net_connection_models.count() as u64;
        }

        for net_event in net_events.read(&mut self.reader) {
            let (event, response) = self.process_connection_event(
                &net_event,
                &entities,
                &mut net_connection_models,
                &game_time_service,
            );

            if let Some(event) = event {
                connection_events.0.push(event);
            }
            if let Some(response) = response {
                let addr = event_peer_addr(&net_event)
                    .expect("Expected to respond to an event with SocketAddr");
                transport.send_with_requirements(
                    addr,
                    &response,
                    DeliveryRequirement::Unreliable,
                    UrgencyRequirement::Immediate,
                );
            }
        }

        for connection_model in (&mut net_connection_models).join() {
            if connection_model.disconnected {
                continue;
            }

            if connection_model.ping_pong_data.last_pinged_at
                + Duration::from_millis(PING_INTERVAL_MILLIS)
                < Instant::now()
            {
                connection_model
                    .ping_pong_data
                    .add_ping(ping_id, game_time_service.engine_time().frame_number());
                transport.send_with_requirements(
                    connection_model.addr,
                    &ping_message(connection_model.session_id, ping_id),
                    DeliveryRequirement::Unreliable,
                    UrgencyRequirement::Immediate,
                );
            }
        }
    }
}

fn ping_message(session_id: NetIdentifier, ping_id: NetIdentifier) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage {
        session_id,
        payload: OutcomingMessagePayload::Ping(ping_id),
    })
    .expect("Expected to serialize Ping message")
    .into()
}

fn pong_message(
    session_id: NetIdentifier,
    ping_id: NetIdentifier,
    frame_number: u64,
) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage {
        session_id,
        payload: OutcomingMessagePayload::Pong {
            ping_id,
            frame_number,
        },
    })
    .expect("Expected to serialize Pong message")
    .into()
}

impl NetConnectionManagerSystem {
    fn process_connection_event(
        &mut self,
        event: &NetworkSimulationEvent,
        entities: &Entities,
        net_connection_models: &mut WriteStorage<NetConnectionModel>,
        game_time_service: &GameTimeService,
    ) -> (
        Option<ConnectionNetEvent<IncomingMessage>>,
        Option<EncodedMessage>,
    ) {
        let peer_addr = event_peer_addr(event);
        if peer_addr.is_none() {
            return (None, None);
        }
        let peer_addr = peer_addr.unwrap();

        if let NetworkSimulationEvent::Connect(socket_addr) = event {
            log::info!("Detected a new UDP connection: {}", socket_addr);
            return (None, None);
        }

        let mut connection = (entities, &mut *net_connection_models)
            .join()
            .find(|(_, connection_model)| connection_model.addr == peer_addr);
        if connection.is_none() {
            if let NetworkSimulationEvent::Disconnect(_) = event {
                log::trace!("Ignoring Disconnect event for an already dropped connection");
                return (None, None);
            }

            let connection_id = self.next_connection_id();
            log::info!(
                "Creating a new NewConnectionModel ({}) for {}",
                connection_id,
                peer_addr
            );
            let net_connection_model = NetConnectionModel::new(connection_id, 0, peer_addr);
            let entity = entities
                .build_entity()
                .with(net_connection_model, net_connection_models)
                .build();
            connection = Some((entity, net_connection_models.get_mut(entity).unwrap()))
        }
        let (connection_model_entity, connection_model) = connection.unwrap();

        let connection_id = connection_model.id;
        match event {
            NetworkSimulationEvent::Disconnect(_) => {
                log::info!(
                    "Dropping a connection ({}) to {}...",
                    connection_model.id,
                    connection_model.addr,
                );
                connection_model.disconnected = true;
                entities
                    .delete(connection_model_entity)
                    .expect("Expected to delete a NetConnectionModel");
                (
                    Some(ConnectionNetEvent {
                        connection_id,
                        event: NetEvent::Disconnected,
                    }),
                    None,
                )
            }
            NetworkSimulationEvent::Message(_, bytes) => {
                if let Ok(IncomingMessage {
                    session_id,
                    payload,
                }) = bincode::deserialize::<IncomingMessage>(bytes.as_ref())
                {
                    match payload {
                        IncomingMessagePayload::Ping(ping_id) => {
                            log::trace!("Received a new ping message: {:?}", &payload);
                            if connection_model.disconnected {
                                return (None, None);
                            }
                            (
                                None,
                                Some(pong_message(
                                    session_id,
                                    ping_id,
                                    game_time_service.game_frame_number(),
                                )),
                            )
                        }
                        IncomingMessagePayload::Pong {
                            ping_id,
                            frame_number: peer_frame_number,
                        } => {
                            log::trace!("Received a new pong message: {:?}", &payload);
                            connection_model.ping_pong_data.add_pong(
                                ping_id,
                                peer_frame_number,
                                game_time_service.engine_time().frame_number(),
                                game_time_service.game_frame_number(),
                            );
                            (None, None)
                        }
                        message if message.is_heartbeat() => {
                            log::trace!(
                                "Received a new Heartbeat message (connection_id: {})",
                                connection_id
                            );
                            (None, None)
                        }
                        _ => {
                            log::debug!(
                                "Received a new message (connection_id: {}): {:?}",
                                connection_id,
                                &payload
                            );
                            (
                                Some(ConnectionNetEvent {
                                    connection_id,
                                    event: NetEvent::Message(IncomingMessage {
                                        session_id,
                                        payload,
                                    }),
                                }),
                                None,
                            )
                        }
                    }
                } else {
                    (None, None)
                }
            }
            NetworkSimulationEvent::SendError(err, _) => {
                log::error!("(SendError) {:?}", err);
                (None, None)
            }
            NetworkSimulationEvent::RecvError(err) => {
                log::error!("(RecvError) {:?}", err);
                (None, None)
            }
            NetworkSimulationEvent::ConnectionError(err, _) => {
                log::error!("(ConnectionError) {:?}", err);
                (None, None)
            }
            _ => (None, None),
        }
    }
}

fn event_peer_addr(event: &NetworkSimulationEvent) -> Option<SocketAddr> {
    match event {
        NetworkSimulationEvent::Connect(addr)
        | NetworkSimulationEvent::Disconnect(addr)
        | NetworkSimulationEvent::Message(addr, _) => Some(*addr),
        NetworkSimulationEvent::ConnectionError(_, addr) => *addr,
        _ => None,
    }
}
