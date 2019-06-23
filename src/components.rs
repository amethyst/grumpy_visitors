use amethyst::ecs::prelude::{Component, DenseVecStorage, VecStorage};
use shrinkwraprs::Shrinkwrap;

use std::time::Duration;

use crate::{
    models::mob_actions::MobAction,
    models::{common::MissileTarget, player_actions::*},
    Vector2, ZeroVector,
};

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
    pub target: MissileTarget,
    pub velocity: Vector2,
    pub acceleration: f32,
    pub time_spawned: Duration,
    pub damage: f32,
}

impl Missile {
    pub fn new(target: MissileTarget, direction: Vector2, time_spawned: Duration) -> Self {
        Self {
            target,
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
            velocity: Vector2::zero(),
            walking_direction: Vector2::new(0.0.into(), 1.0.into()),
            looking_direction: Vector2::new(0.0.into(), 1.0.into()),
            radius: 20.0,
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

pub struct PlayerActions {
    pub walk_actions: Vec<PlayerWalkAction>,
    pub look_actions: Vec<PlayerLookAction>,
    pub cast_actions: Vec<PlayerCastAction>,
    pub last_spell_cast: Duration,
}

impl PlayerActions {
    pub fn new() -> Self {
        Self {
            walk_actions: Vec::new(),
            look_actions: Vec::new(),
            cast_actions: Vec::new(),
            last_spell_cast: Duration::new(0, 0),
        }
    }
}

impl Component for PlayerActions {
    type Storage = DenseVecStorage<Self>;
}

pub struct Monster {
    pub health: f32,
    pub destination: Vector2,
    pub velocity: Vector2,
    pub action: MobAction,
    pub name: String,
    pub radius: f32,
}

impl Component for Monster {
    type Storage = DenseVecStorage<Self>;
}
