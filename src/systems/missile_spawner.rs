use amethyst::{
    assets::Handle,
    core::{Time, Transform},
    ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage},
    renderer::{Material, Mesh},
};

use std::time::Duration;

use crate::{
    components::{Missile, Monster, PlayerActions, WorldPosition},
    data_resources::{EntityGraphics, MissileGraphics},
    models::common::MissileTarget,
    utils::world::closest_monster,
};

pub struct MissileSpawnerSystem;

const SPELL_CAST_COOLDOWN: Duration = Duration::from_millis(500);

impl<'s> System<'s> for MissileSpawnerSystem {
    type SystemData = (
        ReadExpect<'s, Time>,
        ReadExpect<'s, MissileGraphics>,
        Entities<'s>,
        ReadStorage<'s, Monster>,
        WriteStorage<'s, PlayerActions>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Mesh>>,
        WriteStorage<'s, Handle<Material>>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, Missile>,
    );

    fn run(
        &mut self,
        (
            time,
            missile_graphic,
            entities,
            monsters,
            mut player_actions,
            mut transforms,
            mut meshes,
            mut materials,
            mut world_positions,
            mut missiles,
        ): Self::SystemData,
    ) {
        let now = time.absolute_time();
        let EntityGraphics { mesh, material } = missile_graphic.0.clone();

        for player_actions in (&mut player_actions).join() {
            for cast_action in player_actions.cast_actions.drain(..) {
                if player_actions.last_spell_cast + SPELL_CAST_COOLDOWN > now {
                    continue;
                }
                player_actions.last_spell_cast = now;
                let mut transform = Transform::default();
                transform.set_translation_xyz(
                    cast_action.cast_position.x,
                    cast_action.cast_position.y,
                    0.0,
                );

                let search_result = closest_monster(
                    cast_action.target_position,
                    &world_positions,
                    &entities,
                    &monsters,
                );

                let (target, direction) = if let Some((monster, monster_position)) = search_result {
                    (
                        MissileTarget::Target(monster),
                        monster_position - cast_action.cast_position,
                    )
                } else {
                    (
                        MissileTarget::Destination(cast_action.target_position),
                        cast_action.target_position - cast_action.cast_position,
                    )
                };

                entities
                    .build_entity()
                    .with(mesh.clone(), &mut meshes)
                    .with(material.clone(), &mut materials)
                    .with(transform, &mut transforms)
                    .with(
                        WorldPosition::new(cast_action.cast_position),
                        &mut world_positions,
                    )
                    .with(Missile::new(5.0, target, direction, now), &mut missiles)
                    .build();
            }
        }
    }
}
