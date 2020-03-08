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
            mut incoming_messages,
            mut net_connection_models,
            entities,
        ): Self::SystemData,
    ) {
        let ping_id = self.next_ping_id();
        let send_ping_packet = ping_message(ping_id);

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
                incoming_messages.0.push(event);
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
            if connection_model.ping_pong_data.last_pinged_at
                + Duration::from_millis(PING_INTERVAL_MILLIS)
                < Instant::now()
            {
                connection_model
                    .ping_pong_data
                    .add_ping(ping_id, game_time_service.engine_time().frame_number());
                transport.send_with_requirements(
                    connection_model.addr,
                    &send_ping_packet,
                    DeliveryRequirement::Unreliable,
                    UrgencyRequirement::Immediate,
                );
            }
        }
    }
}

fn ping_message(ping_id: NetIdentifier) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage::Ping(ping_id))
        .expect("Expected to serialize Ping message")
        .into()
}

fn pong_message(ping_id: NetIdentifier, frame_number: u64) -> EncodedMessage {
    bincode::serialize(&OutcomingMessage::Pong {
        ping_id,
        frame_number,
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
            let connection_id = self.next_connection_id();
            log::info!(
                "Creating a new NewConnectionModel ({}) for {}",
                connection_id,
                peer_addr
            );
            let net_connection_model = NetConnectionModel::new(connection_id, peer_addr);
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
            NetworkSimulationEvent::SendError(err, _) => {
                log::error!("{:?}", err);
                (None, None)
            }
            NetworkSimulationEvent::RecvError(err) => {
                log::error!("{:?}", err);
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
