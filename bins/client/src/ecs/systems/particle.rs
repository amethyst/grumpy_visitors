use amethyst::{
    core::Transform,
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
};

use gv_client_shared::ecs::components::SpellParticle;
use gv_core::ecs::{
    components::{missile::Missile, WorldPosition},
    system_data::time::GameTimeService,
};

pub struct ParticleSystem;

impl<'s> System<'s> for ParticleSystem {
    type SystemData = (
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Missile>,
        WriteStorage<'s, SpellParticle>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            entities,
            missiles,
            mut spell_particles,
            mut world_positions,
            mut transforms,
        ): Self::SystemData,
    ) {
        let frame_number = game_time_service.game_frame_number();
        for (missile_entity, _missile) in (&entities, &missiles).join() {
            let missile_position = world_positions
                .get(missile_entity)
                .expect("Expected WorldPosition for a missile")
                .clone();
            let mut transform = Transform::default();
            transform.set_translation_xyz(missile_position.x, missile_position.y, 50.0);

            entities
                .build_entity()
                .with(
                    SpellParticle {
                        frame_spawned: frame_number,
                    },
                    &mut spell_particles,
                )
                .with(transform, &mut transforms)
                .with(missile_position, &mut world_positions)
                .build();
        }

        for (spell_particle_entity, spell_particle) in (&entities, &spell_particles).join() {
            if game_time_service.seconds_to_frame(spell_particle.frame_spawned) > 0.15 {
                entities
                    .delete(spell_particle_entity)
                    .expect("Expected to delete SpellParticle");
            }
        }
    }
}
