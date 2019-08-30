mod action_subsystem;
mod dying;

pub use self::{
    action_subsystem::{ApplyWalkActionNetArgs, PlayerActionSubsystem},
    dying::PlayerDyingSystem,
};
