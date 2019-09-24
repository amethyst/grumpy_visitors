pub mod mob;
pub mod monster_spawn;
pub mod player;

use serde_derive::{Deserialize, Serialize};

use crate::net::NetIdentifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action<T> {
    /// Frame number when action was created.
    pub frame_number: u64,
    /// Action payload.
    pub action: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientActionUpdate<T> {
    /// Client generated ID, is needed for clients to check if any particular action was modified
    /// or discarded by server.
    pub client_action_id: NetIdentifier,
    /// Action payload.
    pub action: T,
}
