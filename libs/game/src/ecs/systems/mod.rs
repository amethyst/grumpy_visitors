pub mod missile;
pub mod monster;
pub mod player;

mod action;
mod level;
mod net_connection_manager;
mod pause;
mod state_switcher;
mod world_position_transform;
mod world_state_subsystem;

pub use self::{
    action::ActionSystem, level::LevelSystem, net_connection_manager::NetConnectionManagerSystem,
    pause::PauseSystem, state_switcher::StateSwitcherSystem,
    world_position_transform::WorldPositionTransformSystem,
    world_state_subsystem::WorldStateSubsystem,
};

use amethyst::ecs::{WriteExpect, WriteStorage};
#[cfg(feature = "client")]
use amethyst::{
    assets::Handle,
    ecs::ReadExpect,
    renderer::{Material, Mesh},
};

#[cfg(not(feature = "client"))]
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "client")]
use ha_client_shared::ecs::resources::MissileGraphics;
#[cfg(feature = "client")]
use ha_core::ecs::resources::world::{
    ClientWorldUpdates, PlayerActionUpdates, ReceivedServerWorldUpdate,
};
#[cfg(not(feature = "client"))]
use ha_core::ecs::resources::world::{
    DummyFramedUpdate, ReceivedClientActionUpdates, ServerWorldUpdate, ServerWorldUpdates,
};

#[cfg(feature = "client")]
pub type AggregatedOutcomingUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
pub type AggregatedOutcomingUpdates = ServerWorldUpdates;

#[cfg(feature = "client")]
pub type OutcomingNetUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
pub type OutcomingNetUpdates = ServerWorldUpdate;

#[cfg(feature = "client")]
type ClientFrameUpdate = PlayerActionUpdates;
#[cfg(not(feature = "client"))]
type ClientFrameUpdate = DummyFramedUpdate;

#[cfg(feature = "client")]
type FrameUpdate = ReceivedServerWorldUpdate;
#[cfg(not(feature = "client"))]
type FrameUpdate = ReceivedClientActionUpdates;

type WriteStorageCell<'s, T> = Rc<RefCell<WriteStorage<'s, T>>>;
type WriteExpectCell<'s, T> = Rc<RefCell<WriteExpect<'s, T>>>;

#[cfg(feature = "client")]
pub struct GraphicsResourceBundle<'s> {
    missile_graphics: ReadExpect<'s, MissileGraphics>,
    meshes: WriteStorageCell<'s, Handle<Mesh>>,
    materials: WriteStorageCell<'s, Handle<Material>>,
}

#[cfg(not(feature = "client"))]
pub struct GraphicsResourceBundle<'s> {
    _lifetime: PhantomData<&'s ()>,
}
