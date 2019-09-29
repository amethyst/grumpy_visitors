mod action_subsystem;
mod dying;

pub use self::{
    action_subsystem::{ApplyLookActionNetArgs, ApplyWalkActionNetArgs, PlayerActionSubsystem},
    dying::PlayerDyingSystem,
};
