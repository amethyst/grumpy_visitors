pub mod missile;
pub mod monster;
pub mod player;

mod action;
mod damage_subsystem;
mod level;
mod net_connection_manager;
mod pause;
mod state_switcher;
mod world_position_transform;
mod world_state_subsystem;

pub use self::{
    action::ActionSystem,
    damage_subsystem::DamageSubsystem,
    level::LevelSystem,
    net_connection_manager::{NetConnectionManagerDesc, NetConnectionManagerSystem},
    pause::PauseSystem,
    state_switcher::StateSwitcherSystem,
    world_position_transform::WorldPositionTransformSystem,
    world_state_subsystem::WorldStateSubsystem,
};

use amethyst::ecs::{
    shred::{ResourceId, SystemData},
    Entity, World, WriteExpect, WriteStorage,
};
#[cfg(feature = "client")]
use amethyst::{
    animation::{AnimationControlSet, AnimationSet},
    assets::Handle,
    core::{Named, ParentHierarchy},
    ecs::{ReadExpect, ReadStorage},
    renderer::{sprite::SpriteRender, Material, Mesh},
};

#[cfg(not(feature = "client"))]
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc};

use gv_animation_prefabs::AnimationId;
#[cfg(feature = "client")]
use gv_core::ecs::resources::world::{
    ClientWorldUpdates, PlayerActionUpdates, ReceivedServerWorldUpdate,
};
#[cfg(not(feature = "client"))]
use gv_core::ecs::resources::world::{
    DummyFramedUpdate, ReceivedClientActionUpdates, ServerWorldUpdate, ServerWorldUpdates,
};

#[cfg(feature = "client")]
use crate::utils::entities::{play_animation, remove_animation};

#[cfg(feature = "client")]
pub type AggregatedOutcomingUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
pub type AggregatedOutcomingUpdates = ServerWorldUpdates;

#[cfg(feature = "client")]
pub type OutcomingNetUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
pub type OutcomingNetUpdates = ServerWorldUpdate;

#[cfg(feature = "client")]
pub type ClientFrameUpdate = PlayerActionUpdates;
#[cfg(not(feature = "client"))]
pub type ClientFrameUpdate = DummyFramedUpdate;

#[cfg(feature = "client")]
pub type FrameUpdate = ReceivedServerWorldUpdate;
#[cfg(not(feature = "client"))]
pub type FrameUpdate = ReceivedClientActionUpdates;

type WriteStorageCell<'s, T> = Rc<RefCell<WriteStorage<'s, T>>>;
type WriteExpectCell<'s, T> = Rc<RefCell<WriteExpect<'s, T>>>;

#[cfg(feature = "client")]
#[allow(dead_code)]
pub struct GraphicsResourceBundle<'s> {
    meshes: WriteStorageCell<'s, Handle<Mesh>>,
    materials: WriteStorageCell<'s, Handle<Material>>,
}

#[cfg(not(feature = "client"))]
pub struct GraphicsResourceBundle<'s> {
    _lifetime: PhantomData<&'s ()>,
}

#[cfg(feature = "client")]
#[derive(SystemData)]
pub struct AnimationsSystemData<'s> {
    pub parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    pub named: ReadStorage<'s, Named>,
    pub animation_sets: ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
    pub animation_control_sets: WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
}

#[cfg(not(feature = "client"))]
#[derive(SystemData)]
pub struct AnimationsSystemData<'s> {
    _lifetime: PhantomData<&'s ()>,
}

impl<'s> AnimationsSystemData<'s> {
    #[cfg(feature = "client")]
    fn play_animation(&mut self, entity: Entity, body_part_name: &str, animation_id: AnimationId) {
        play_animation(
            &self.parent_hierarchy,
            &self.named,
            &self.animation_sets,
            &mut self.animation_control_sets,
            entity,
            body_part_name,
            animation_id,
        );
    }

    #[cfg(not(feature = "client"))]
    fn play_animation(
        &mut self,
        _entity: Entity,
        _body_part_name: &str,
        _animation_id: AnimationId,
    ) {
    }

    #[cfg(feature = "client")]
    fn remove_animation(
        &mut self,
        entity: Entity,
        body_part_name: &str,
        animation_id: AnimationId,
    ) {
        remove_animation(
            &self.parent_hierarchy,
            &self.named,
            &mut self.animation_control_sets,
            entity,
            body_part_name,
            animation_id,
        );
    }

    #[cfg(not(feature = "client"))]
    fn remove_animation(
        &mut self,
        _entity: Entity,
        _body_part_name: &str,
        _animation_id: AnimationId,
    ) {
    }
}

#[cfg(feature = "client")]
pub struct AnimationsResourceBundle<'s> {
    pub parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    pub named: ReadStorage<'s, Named>,
    pub animation_sets: ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
    pub animation_control_sets:
        WriteStorageCell<'s, AnimationControlSet<AnimationId, SpriteRender>>,
}

#[cfg(not(feature = "client"))]
pub struct AnimationsResourceBundle<'s> {
    _lifetime: PhantomData<&'s ()>,
}

impl<'s> AnimationsResourceBundle<'s> {
    #[cfg(feature = "client")]
    fn play_animation(&self, entity: Entity, body_part_name: &str, animation_id: AnimationId) {
        play_animation(
            &self.parent_hierarchy,
            &self.named,
            &self.animation_sets,
            &mut self.animation_control_sets.borrow_mut(),
            entity,
            body_part_name,
            animation_id,
        );
    }

    #[cfg(not(feature = "client"))]
    fn play_animation(&self, _entity: Entity, _body_part_name: &str, _animation_id: AnimationId) {}
}
