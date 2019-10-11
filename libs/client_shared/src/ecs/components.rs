use amethyst::ecs::{Component, DenseVecStorage};

use gv_core::math::Vector2;

pub struct HealthUiGraphics {
    pub screen_position: Vector2,
    pub scale_ratio: f32,
    pub health: f32,
}

impl Component for HealthUiGraphics {
    type Storage = DenseVecStorage<Self>;
}
