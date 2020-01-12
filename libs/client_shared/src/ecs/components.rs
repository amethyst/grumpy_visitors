use amethyst::ecs::{Component, DenseVecStorage, NullStorage};

use gv_core::math::Vector2;

#[derive(Component)]
pub struct HealthUiGraphics {
    pub screen_position: Vector2,
    pub scale_ratio: f32,
    pub health: f32,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct MagePreviewCamera;

#[derive(Component)]
pub struct MagePreview {
    pub color: [f32; 4],
}
