use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

use std::time::Duration;

use ha_core::math::Vector2;

pub struct Missile {
    pub radius: f32,
    pub target: MissileTarget,
    pub velocity: Vector2,
    pub time_spawned: Duration,
    pub damage: f32,
}

impl Missile {
    pub fn new(
        radius: f32,
        target: MissileTarget,
        velocity: Vector2,
        time_spawned: Duration,
    ) -> Self {
        Self {
            radius,
            target,
            velocity,
            time_spawned,
            damage: 50.0,
        }
    }
}

impl Component for Missile {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone)]
pub enum MissileTarget {
    Target(Entity),
    Destination(Vector2),
}
