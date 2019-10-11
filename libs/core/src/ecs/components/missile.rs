use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

use crate::{math::Vector2, net::NetIdentifier};

#[derive(Clone, Debug)]
pub struct Missile {
    pub action_id: NetIdentifier,
    pub radius: f32,
    pub target: MissileTarget<Entity>,
    pub velocity: Vector2,
    pub frame_spawned: u64,
    pub damage: f32,
}

impl Missile {
    pub fn new(
        action_id: u64,
        radius: f32,
        target: MissileTarget<Entity>,
        velocity: Vector2,
        frame_spawned: u64,
    ) -> Self {
        Self {
            action_id,
            radius,
            target,
            velocity,
            frame_spawned,
            damage: 50.0,
        }
    }
}

impl Component for Missile {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Debug)]
pub enum MissileTarget<T> {
    Target(T),
    Destination(Vector2),
}
