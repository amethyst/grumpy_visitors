use amethyst::{
    renderer::{
        rendy::resource::{Handle, Image},
        Backend,
    },
    Error,
};

use std::{
    env::current_exe,
    net::SocketAddr,
    process::{Child, Command},
};

pub struct LastAcknowledgedUpdate {
    pub id: u64,
    pub frame_number: u64,
}

pub struct ServerCommand {
    process: Option<ServerProcess>,
}

impl ServerCommand {
    pub fn new() -> Self {
        Self { process: None }
    }

    pub fn start(&mut self, addr: SocketAddr, host_client_addr: SocketAddr) -> Result<(), Error> {
        self.process = Some(ServerProcess::new(addr, Some(host_client_addr))?);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn kill(&mut self) {
        self.process = None;
    }
}

pub struct ServerProcess {
    cmd: Child,
}

impl ServerProcess {
    pub fn new(addr: SocketAddr, host_client_addr: Option<SocketAddr>) -> Result<Self, Error> {
        let executable_path = {
            let mut path = current_exe()?;
            path.pop();
            path.join("gv_server")
        };

        let mut command_builder = Command::new(executable_path);
        command_builder.arg("--addr").arg(addr.to_string());

        if let Some(host_client_addr) = host_client_addr {
            command_builder
                .arg("--client-addr")
                .arg(host_client_addr.to_string());
        }

        let cmd = command_builder.spawn()?;

        Ok(ServerProcess { cmd })
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        if self.cmd.kill().is_err() {
            log::warn!(
                "Tried to kill the ServerProcess (id: {}) which wasn't running",
                self.cmd.id()
            );
        }
    }
}

pub struct DrawMagePreviewTargetImage<B: Backend>(pub Option<Handle<Image<B>>>);
