use amethyst::ecs::prelude::{Component, DenseVecStorage, VecStorage};
use shrinkwraprs::Shrinkwrap;

use std::time::Duration;

use crate::{models::MonsterAction, Vector2};

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct WorldPosition {
    #[shrinkwrap(main_field)]
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
    pub time_spawned: Duration,
    pub damage: f32,
}

impl Missile {
    pub fn new(direction: Vector2, time_spawned: Duration) -> Self {
        Self {
            velocity: direction,
            acceleration: 10.0,
            time_spawned,
            damage: 50.0,
        }
    }
}

impl Component for Missile {
    type Storage = DenseVecStorage<Self>;
}

pub struct Player {
    pub velocity: Vector2,
    pub walking_direction: Vector2,
    pub looking_direction: Vector2,
    pub radius: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            velocity: Vector2::new(0.0.into(), 0.0.into()),
            walking_direction: Vector2::new(0.0.into(), 1.0.into()),
            looking_direction: Vector2::new(0.0.into(), 1.0.into()),
            radius: 20.0,
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

pub struct Monster {
    pub health: f32,
    pub destination: Vector2,
    pub name: String,
    pub action: MonsterAction,
}

impl Component for Monster {
    type Storage = DenseVecStorage<Self>;
}
