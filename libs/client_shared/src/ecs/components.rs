use amethyst::ecs::{Component, DenseVecStorage, NullStorage};

use ha_core::math::Vector2;

pub struct HealthUiGraphics {
    pub screen_position: Vector2,
    pub scale_ratio: f32,
    pub health: f32,
}

impl Component for HealthUiGraphics {
    type Storage = DenseVecStorage<Self>;
}

pub struct ControllablePlayer;

impl Component for ControllablePlayer {
    type Storage = NullStorage<Self>;
}

impl Default for ControllablePlayer {
    fn default() -> Self {
        Self
    }
}
