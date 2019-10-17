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
    action::ActionSystem, damage_subsystem::DamageSubsystem, level::LevelSystem,
    net_connection_manager::NetConnectionManagerSystem, pause::PauseSystem,
    state_switcher::StateSwitcherSystem, world_position_transform::WorldPositionTransformSystem,
    world_state_subsystem::WorldStateSubsystem,
};

use amethyst::ecs::{Entity, WriteExpect, WriteStorage};
#[cfg(feature = "client")]
use amethyst::{
    animation::{AnimationCommand, AnimationControlSet, AnimationSet, EndControl},
    assets::Handle,
    core::{Named, ParentHierarchy},
    ecs::{Read, ReadExpect, ReadStorage},
    renderer::{sprite::SpriteRender, Material, Mesh},
};

#[cfg(not(feature = "client"))]
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc};

use gv_animation_prefabs::AnimationId;
#[cfg(feature = "client")]
use gv_client_shared::ecs::resources::MissileGraphics;
#[cfg(feature = "client")]
use gv_core::ecs::resources::world::{
    ClientWorldUpdates, PlayerActionUpdates, ReceivedServerWorldUpdate,
};
#[cfg(not(feature = "client"))]
use gv_core::ecs::resources::world::{
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
    missile_graphics: Option<Read<'s, MissileGraphics>>,
    meshes: WriteStorageCell<'s, Handle<Mesh>>,
    materials: WriteStorageCell<'s, Handle<Material>>,
}

#[cfg(not(feature = "client"))]
pub struct GraphicsResourceBundle<'s> {
    _lifetime: PhantomData<&'s ()>,
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
        let body_part_entity = self
            .parent_hierarchy
            .children(entity)
            .iter()
            .find(|child_entity| {
                if let Some(entity_name) = self.named.get(**child_entity) {
                    if entity_name.name == body_part_name {
                        return true;
                    }
                }
                false
            });
        if body_part_entity.is_none() {
            log::warn!(
                "Couldn't find the body part and play an animation: {}",
                body_part_name
            );
            return;
        }
        let body_part_entity = body_part_entity.unwrap();

        let mut animation_control_sets = self.animation_control_sets.borrow_mut();
        let animation_control_set = animation_control_sets.get_mut(*body_part_entity);
        if let Some(animation_control_set) = animation_control_set {
            let animation_set = self
                .animation_sets
                .get(*body_part_entity)
                .expect("Expected AnimationSet for an entity with AnimationControlSet");
            animation_control_set.add_animation(
                animation_id,
                &animation_set.get(&animation_id).unwrap(),
                EndControl::Stay,
                1.0,
                AnimationCommand::Start,
            );
        }
    }

    #[cfg(not(feature = "client"))]
    fn play_animation(&self, _entity: Entity, _body_part_name: &str, _animation_id: AnimationId) {}
}
