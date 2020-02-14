mod dying;
mod physics_subsystem;
mod spawner_subsystem;

pub use self::{
    dying::{MissileDyingSystem, MISSILE_TTL_SECS},
    physics_subsystem::{
        MissilePhysicsSubsystem, MISSILE_LIFESPAN_SECS, MISSILE_MAX_SPEED, MISSILE_MIN_SPEED,
        MISSILE_TIME_TO_FADE,
    },
    spawner_subsystem::{MissileFactory, MissileSpawnerSubsystem},
};
