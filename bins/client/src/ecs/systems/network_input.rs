//use amethyst::{
//    ecs::{Entities, Join, System, WriteExpect, WriteStorage},
//    network::ConnectionState,
//};
//
//use ha_core::net::NetConnection;
//
//use crate::ecs::resources::ServerCommand;
//
pub struct NetworkInputSystem;
//
//impl<'s> System<'s> for NetworkInputSystem {
//    type SystemData = (
//        Entities<'s>,
//        WriteExpect<'s, ServerCommand>,
//        WriteStorage<'s, NetConnection>,
//    );
//
//    fn run(&mut self, (entities, mut server_command, mut net_connections): Self::SystemData) {
//        let server_process = server_command.process();
//        if server_process.is_none() {
//            return;
//        }
//        let server_process = server_process.unwrap();
//
//        server_process.socket_addr();
//    }
//}
//
//impl NetworkInputSystem {
//    fn connect(
//        &mut self,
//        entities: Entities,
//        server_command: &mut WriteExpect<ServerCommand>,
//        net_connections: &mut WriteStorage<NetConnection>,
//    ) -> bool {
//        let (had_connection, is_alive, is_connected) = {
//            // Check if we have NetConnection component already,
//            // drop server process if we got disconnected.
//            if let Some((entity, net_connection)) = (&entities, net_connections).join().next() {
//                match net_connection.state {
//                    ConnectionState::Disconnected => {
//                        if server_command.process().is_some() {
//                            server_command.kill();
//                        }
//                        (server_command.process().is_some(), false, false)
//                    }
//                    ConnectionState::Connected => (true, true, true),
//                    ConnectionState::Connecting => (true, true, false),
//                }
//            } else {
//                (false, false, false)
//            }
//        };
//
//        if had_connection {
//            return is_alive && is_connected;
//        }
//
//        true
//    }
//}
