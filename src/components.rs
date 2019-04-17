use amethyst::ecs::prelude::{Component, DenseVecStorage, VecStorage};

use std::time::Instant;

use crate::{models::MonsterAction, Vector2};

pub struct WorldPosition {
    pub position: Vector2,
}

impl WorldPosition {
    pub fn new(position: Vector2) -> Self {
        Self { position }
    }
}

impl Component for WorldPosition {
    type Storage = VecStorage<Self>;
}

pub struct Missile {
    pub velocity: Vector2,
    pub acceleration: f32,
    pub time_spawned: Instant,
}

impl Missile {
    pub fn new(direction: Vector2, time_spawned: Instant) -> Self {
        Self {
            velocity: direction,
            acceleration: 10.0,
            time_spawned,
        }
    }
}

impl Component for Missile {
    type Storage = DenseVecStorage<Self>;
}

pub struct Player {
    pub velocity: Vector2,
    pub radius: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            velocity: Vector2::new(0.0, 0.0),
            radius: 20.0,
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

pub struct Monster {
    pub health: f32,
    pub velocity: Vector2,
    pub name: String,
    pub action: MonsterAction,
}

impl Component for Monster {
    type Storage = DenseVecStorage<Self>;
}
