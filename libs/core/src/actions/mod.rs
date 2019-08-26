use serde_derive::{Deserialize, Serialize};

pub mod mob;
pub mod monster_spawn;
pub mod player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action<T> {
    pub frame_number: u64,
    pub action: Option<T>,
}

impl<T> Default for Action<T> {
    fn default() -> Self {
        Self {
            frame_number: 0,
            action: None,
        }
    }
}