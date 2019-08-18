use amethyst::{
    ecs::{Entities, Join, System, WriteStorage},
    network::{NetEvent, NetPacket},
};
use log;

use std::time::{Duration, Instant};

#[cfg(feature = "client")]
use ha_core::net::client_messages::ClientMessages;
#[cfg(not(feature = "client"))]
use ha_core::net::server_messages::ServerMessages;
use ha_core::{ecs::components::NetConnectionModel, net::NetConnection};

const PING_INTERVAL_SECS: u64 = 1;

pub struct NetConnectionManagerSystem;

impl<'s> System<'s> for NetConnectionManagerSystem {
    type SystemData = (
        WriteStorage<'s, NetConnection>,
        WriteStorage<'s, NetConnectionModel>,
        Entities<'s>,
    );

    fn run(&mut self, (mut connections, mut connection_readers, entities): Self::SystemData) {
        let mut count = 0;
        let mut connection_count = 0;

        let send_ping_packet = NetEvent::Packet(NetPacket::unreliable(ping_message()));

        for (e, connection) in (&entities, &mut connections).join() {
            let mut connection_messages_count = 0;
            let connection_data = connection_readers
                .entry(e)
                .expect("Expected to get the right generation")
                .or_insert_with(|| NetConnectionModel::new(connection.register_reader()));

            let mut client_disconnected = false;

            for ev in connection.received_events(&mut connection_data.reader) {
                connection_messages_count += 1;
                match ev {
                    NetEvent::Packet(packet) => log::debug!("{:?}", packet.content()),
                    NetEvent::Connected(addr) => log::info!("New client connection: {}", addr),
                    NetEvent::Disconnected(_addr) => {
                        client_disconnected = true;
                    }
                    _ => {}
                }
            }

            if connection_data.last_pinged_at + Duration::from_secs(PING_INTERVAL_SECS)
                < Instant::now()
            {
                connection_data.last_pinged_at = Instant::now();
                connection.queue(send_ping_packet.clone());
            }

            if client_disconnected {
                log::info!("Dropping a connection to {}...", connection.target_addr);
                entities
                    .delete(e)
                    .expect("Expected to delete a ConnectionReader");
            }

            if connection_messages_count > 0 {
                connection_count += 1;
                count += connection_messages_count;
            }
        }

        if count > 0 {
            log::info!(
                "Received {} messages this frame from {} connections",
                count,
                connection_count
            );
        }
    }
}

#[cfg(feature = "client")]
fn ping_message() -> Vec<u8> {
    bincode::serialize(&ClientMessages::Ping).expect("Expected to serialize Ping message")
}

#[cfg(not(feature = "client"))]
fn ping_message() -> Vec<u8> {
    bincode::serialize(&ServerMessages::Ping).expect("Expected to serialize Ping message")
}
