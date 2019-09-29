use serde_derive::{Deserialize, Serialize};

use crate::math::Vector2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerWalkAction {
    Walk { direction: Vector2 },
    Stop,
}

impl PartialEq for PlayerWalkAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PlayerWalkAction::Walk { direction },
                PlayerWalkAction::Walk {
                    direction: other_direction,
                },
            ) => (direction - other_direction).norm_squared() < 0.001,
            (PlayerWalkAction::Stop, PlayerWalkAction::Stop) => true,
            _ => false,
        }
    }
}

impl Default for PlayerWalkAction {
    fn default() -> Self {
        Self::Stop
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLookAction {
    pub direction: Vector2,
}

impl PartialEq for PlayerLookAction {
    fn eq(&self, other: &Self) -> bool {
        (self.direction - other.direction).norm_squared() < 0.001
    }
}

impl Default for PlayerLookAction {
    fn default() -> Self {
        Self {
            direction: Vector2::new(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCastAction {
    pub cast_position: Vector2,
    pub target_position: Vector2,
}

impl PartialEq for PlayerCastAction {
    fn eq(&self, other: &Self) -> bool {
        (self.cast_position - other.cast_position).norm_squared() < 0.001
            && (self.target_position - other.target_position).norm_squared() < 0.001
    }
}
