mod dying;
mod physics_subsystem;
mod spawner_subsystem;

pub use self::{
    dying::MissileDyingSystem,
    physics_subsystem::{MissilePhysicsSubsystem, MISSILE_MAX_SPEED, MISSILE_MIN_SPEED},
    spawner_subsystem::{MissileFactory, MissileSpawnerSubsystem},
};
