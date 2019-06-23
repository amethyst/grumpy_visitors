use crate::Vector2;

pub struct PlayerWalkAction {
    pub direction: Vector2,
}

pub struct PlayerLookAction {
    pub direction: Vector2,
}

pub struct PlayerCastAction {
    pub cast_position: Vector2,
    pub target_position: Vector2,
}
