use serde_derive::{Deserialize, Serialize};

use crate::math::Vector2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerWalkAction {
    pub direction: Vector2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLookAction {
    pub direction: Vector2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCastAction {
    pub cast_position: Vector2,
    pub target_position: Vector2,
}
