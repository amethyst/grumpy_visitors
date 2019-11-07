use amethyst::{
    core::Transform,
    ecs::{Entities, Entity, WriteStorage},
};
use gv_core::profile_scope;

#[cfg(feature = "client")]
use gv_client_shared::ecs::resources::EntityGraphics;
use gv_core::{
    actions::IdentifiableAction,
    ecs::{
        components::{missile::*, Dead, Monster, WorldPosition},
        resources::net::CastActionsToExecute,
        system_data::time::GameTimeService,
    },
    math::Vector2,
};

use crate::{
    ecs::{
        system_data::GameStateHelper,
        systems::{
            missile::physics_subsystem::MISSILE_MAX_SPEED, GraphicsResourceBundle, WriteExpectCell,
            WriteStorageCell,
        },
    },
    utils::world::closest_monster,
};

pub struct MissileSpawnerSubsystem<'a, 's> {
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_state_helper: &'s GameStateHelper<'s>,
    pub entities: &'s Entities<'s>,
    pub missile_factory: &'a MissileFactory<'a, 's>,
    pub cast_actions_to_execute: WriteExpectCell<'s, CastActionsToExecute>,
    pub monsters: WriteStorageCell<'s, Monster>,
    pub dead: WriteStorageCell<'s, Dead>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
}

impl<'a, 's> MissileSpawnerSubsystem<'a, 's> {
    pub fn spawn_missiles(&self, frame_number: u64) {
        profile_scope!("MissileSpawnerSubsystem::spawn_missiles");
        let mut world_positions = self.world_positions.borrow_mut();
        let mut cast_actions_to_execute = self.cast_actions_to_execute.borrow_mut();
        let dead = self.dead.borrow();
        let monsters = self.monsters.borrow();

        for cast_action in cast_actions_to_execute.actions.drain(..) {
            let IdentifiableAction {
                action_id,
                action: cast_action,
            } = cast_action;

            let search_result = closest_monster(
                cast_action.target_position,
                &*world_positions,
                &self.entities,
                &*monsters,
                &*dead,
                frame_number,
            );

            let target = if let Some((monster, _)) = search_result {
                MissileTarget::Target(monster)
            } else {
                MissileTarget::Destination(cast_action.target_position)
            };
            let direction = cast_action.target_position - cast_action.cast_position;
            let velocity = direction.normalize() * MISSILE_MAX_SPEED;

            self.missile_factory.create(
                action_id,
                &mut *world_positions,
                5.0,
                target,
                velocity,
                frame_number,
                cast_action.cast_position,
            );
        }
    }
}

pub struct MissileFactory<'a, 's> {
    entities: &'s Entities<'s>,
    transforms: WriteStorageCell<'s, Transform>,
    missiles: WriteStorageCell<'s, Missile>,
    #[cfg_attr(not(feature = "client"), allow(dead_code))]
    graphics_resource_bundle: &'a GraphicsResourceBundle<'s>,
}

impl<'a, 's> MissileFactory<'a, 's> {
    pub fn new(
        entities: &'s Entities<'s>,
        transforms: WriteStorageCell<'s, Transform>,
        missiles: WriteStorageCell<'s, Missile>,
        graphics_resource_bundle: &'a GraphicsResourceBundle<'s>,
    ) -> Self {
        Self {
            entities,
            transforms,
            missiles,
            graphics_resource_bundle,
        }
    }

    #[cfg(feature = "client")]
    pub fn create(
        &self,
        action_id: u64,
        world_positions: &mut WriteStorage<'s, WorldPosition>,
        radius: f32,
        target: MissileTarget<Entity>,
        velocity: Vector2,
        frame_spawned: u64,
        position: Vector2,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 0.0);

        let EntityGraphics { mesh, material } = self
            .graphics_resource_bundle
            .missile_graphics
            .as_ref()
            .unwrap()
            .0
            .clone();

        self.entities
            .build_entity()
            .with(
                mesh.clone(),
                &mut self.graphics_resource_bundle.meshes.borrow_mut(),
            )
            .with(
                material.clone(),
                &mut self.graphics_resource_bundle.materials.borrow_mut(),
            )
            .with(transform, &mut self.transforms.borrow_mut())
            .with(WorldPosition::new(position), world_positions)
            .with(
                Missile::new(action_id, radius, target, velocity, frame_spawned),
                &mut self.missiles.borrow_mut(),
            )
            .build()
    }

    #[cfg(not(feature = "client"))]
    pub fn create(
        &self,
        action_id: u64,
        world_positions: &mut WriteStorage<'s, WorldPosition>,
        radius: f32,
        target: MissileTarget<Entity>,
        velocity: Vector2,
        frame_spawned: u64,
        position: Vector2,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 0.0);

        self.entities
            .build_entity()
            .with(transform, &mut self.transforms.borrow_mut())
            .with(WorldPosition::new(position), world_positions)
            .with(
                Missile::new(action_id, radius, target, velocity, frame_spawned),
                &mut self.missiles.borrow_mut(),
            )
            .build()
    }
}
