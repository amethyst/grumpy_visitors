use amethyst::ecs::{Entities, System, WriteExpect, WriteStorage};

use std::{
    net::{IpAddr, Ipv4Addr},
    time::{Duration, Instant},
};

use ha_core::net::NetConnection;

use crate::ecs::resources::ServerCommand;

pub struct LocalServerSystem;

// TODO: omg, find another way to connect.
const WAIT_FOR_SERVER_TO_START_SECS: u64 = 2;

impl<'s> System<'s> for LocalServerSystem {
    type SystemData = (
        Entities<'s>,
        WriteExpect<'s, ServerCommand>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(&mut self, (entities, server_command, mut net_connections): Self::SystemData) {
        if server_command.process().is_none() {
            return;
        }
        let server_process = server_command.process().unwrap();

        // TODO: do we need a better way to determine whether we're connected to a local server?
        if net_connections.count() != 0 {
            return;
        }

        if server_process.created_at() + Duration::from_secs(WAIT_FOR_SERVER_TO_START_SECS)
            < Instant::now()
        {
            let mut addr = server_process.socket_addr();
            if addr.ip().is_unspecified() {
                addr.set_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            }

            log::info!("Creating a connection to a local server: {}", addr);

            entities
                .build_entity()
                .with(NetConnection::new(addr), &mut net_connections)
                .build();
        }
    }
}
