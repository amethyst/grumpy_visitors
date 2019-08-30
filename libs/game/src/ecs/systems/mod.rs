pub mod missile;
pub mod monster;
pub mod player;

mod action;
mod level;
mod net_connection_manager;
mod state_switcher;
mod world_position_transform;
mod world_state_subsystem;

pub use self::{
    action::ActionSystem, level::LevelSystem, net_connection_manager::NetConnectionManagerSystem,
    state_switcher::StateSwitcherSystem, world_position_transform::WorldPositionTransformSystem,
    world_state_subsystem::WorldStateSubsystem,
};

use amethyst::ecs::WriteStorage;

use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "client")]
use ha_core::ecs::resources::world::ClientWorldUpdates;
#[cfg(not(feature = "client"))]
use ha_core::ecs::resources::world::ServerWorldUpdate;

#[cfg(feature = "client")]
type OutcomingNetUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
type OutcomingNetUpdates = ServerWorldUpdate;

type WriteStorageCell<'s, T> = Rc<RefCell<WriteStorage<'s, T>>>;
