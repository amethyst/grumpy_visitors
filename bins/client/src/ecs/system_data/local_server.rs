//use amethyst::{
//    ecs::{Entities, Entity, Join, WriteExpect},
//    error::Error,
//    network::{ConnectionState, NetEvent, NetPacket},
//};
//use shred_derive::SystemData;
//
//use ha_core::net::{client_messages::ClientMessages, NetConnection};
//
//use crate::ecs::resources::ServerCommand;
//
//#[derive(SystemData)]
//pub struct LocalServer<'a> {
//    entities: Entities<'a>,
//    server_processes: ReadExpect<'a, ServerCommand>,
//    net_connections: WriteStorage<'a, NetConnection>,
//}
//
//impl<'a> LocalServer<'a> {
//    pub fn initialize(&mut self, addr: String) -> Result<(), Error> {
//        let socket_address = addr.parse()?;
//
//        // We don't want to keep several local servers, so we should drop existing one first.
//        if let Some((entity, server_process, net_connection)) = (
//            &self.entities,
//            &self.server_processes,
//            &self.net_connections,
//        )
//            .join()
//            .next()
//        {
//            self.server_processes.remove(entity);
//            self.net_connections.remove(entity);
//        }
//
//        self.entities
//            .build_entity()
//            .with(ServerProcess::new(addr)?, &mut self.server_processes)
//            .build();
//
//        let mut connection = NetConnection::new(socket_address);
//        connection.queue(NetEvent::Packet(NetPacket::reliable_unordered(
//            bincode::serialize(&ClientMessages::JoinRoom {
//                nickname: "Player".to_owned(),
//            })
//            .expect("Expected to serialize a message"),
//        )));
//
//        self.entities
//            .build_entity()
//            .with(
//                NetConnection::new(socket_address),
//                &mut self.net_connections,
//            )
//            .build();
//
//        Ok(())
//    }
//
//
//}
