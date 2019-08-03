mod action;
mod dying;
mod movement;
mod spawner;

pub use self::{
    action::MonsterActionSystem, dying::MonsterDyingSystem, movement::MonsterMovementSystem,
    spawner::MonsterSpawnerSystem,
};
