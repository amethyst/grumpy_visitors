mod action_subsystem;
mod dying;
mod spawner;

pub use self::{
    action_subsystem::{ApplyMonsterActionNetArgs, MonsterActionSubsystem},
    dying::MonsterDyingSystem,
    spawner::MonsterSpawnerSystem,
};
