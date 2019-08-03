use amethyst::{
    ecs::{Entities, Join, System, WriteStorage},
    network::{NetConnection, NetEvent},
};
use log;

use crate::ecs::components::ConnectionReader;

pub struct NetworkingSystem;

impl<'s> System<'s> for NetworkingSystem {
    type SystemData = (
        WriteStorage<'s, NetConnection<Vec<u8>>>,
        WriteStorage<'s, ConnectionReader>,
        Entities<'s>,
    );

    fn run(&mut self, (mut connections, mut connection_readers, entities): Self::SystemData) {
        let mut count = 0;
        let mut connection_count = 0;

        for (e, connection) in (&entities, &mut connections).join() {
            let mut connection_messages_count = 0;
            let reader = connection_readers
                .entry(e)
                .expect("Expected to get a ConnectionReader")
                .or_insert_with(|| ConnectionReader(connection.register_reader()));

            let mut client_disconnected = false;

            for ev in connection.received_events(&mut reader.0) {
                connection_messages_count += 1;
                match ev {
                    NetEvent::Packet(packet) => log::info!("{:?}", packet.content()),
                    NetEvent::Connected(addr) => log::info!("New Client Connection: {}", addr),
                    NetEvent::Disconnected(_addr) => {
                        client_disconnected = true;
                    }
                    _ => {}
                }
            }

            if client_disconnected {
                println!("Client Disconnects");
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
            println!(
                "Received {} messages this frame from {} connections",
                count, connection_count
            );
        }
    }
}
