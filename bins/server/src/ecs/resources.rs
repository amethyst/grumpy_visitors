use std::net::SocketAddr;

pub struct LastBroadcastedFrame(pub u64);

pub struct HostClientAddress(pub Option<SocketAddr>);
