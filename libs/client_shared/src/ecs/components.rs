use amethyst::ecs::{Component, DenseVecStorage};

use gv_core::math::Vector2;

#[derive(Component)]
pub struct HealthUiGraphics {
    pub screen_position: Vector2,
    pub scale_ratio: f32,
    pub health: f32,
}

#[derive(Component)]
pub struct PlayerColor(pub [f32; 3]);

#[derive(Component)]
pub struct SpellParticle {
    pub frame_spawned: u64,
}
