use amethyst::{
    core::{math::Rotation2, Transform},
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
};
use rand::{self, Rng};

use std::f32::consts::PI;

use gv_client_shared::ecs::components::SpellParticle;
use gv_core::{
    ecs::{
        components::{missile::Missile, WorldPosition},
        system_data::time::GameTimeService,
    },
    math::{Vector2, Vector3},
};
use gv_game::ecs::systems::missile::{MISSILE_MAX_SPEED, MISSILE_MIN_SPEED};

const PARTICLE_SPEED: f32 = 230.0;

pub struct ParticleSystem;

impl<'s> System<'s> for ParticleSystem {
    type SystemData = (
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Missile>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, SpellParticle>,
        WriteStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            entities,
            missiles,
            world_positions,
            mut spell_particles,
            mut transforms,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let frame_number = game_time_service.game_frame_number();
        for (missile_entity, missile) in (&entities, &missiles).join() {
            let missile_position = world_positions
                .get(missile_entity)
                .expect("Expected WorldPosition for a missile")
                .clone();
            let mut transform = Transform::default();
            transform.set_translation_xyz(missile_position.x, missile_position.y, 50.0);

            let missile_speed = missile.velocity.norm();
            let particle_velocity = if missile_speed == 0.0 {
                let angle = rng.gen_range(0.0, PI * 2.0);
                Rotation2::new(angle) * Vector2::new(0.0, 1.0) * PARTICLE_SPEED
            } else {
                let min_rotation = PI / 6.0;
                let speed_multiplier = 1.0
                    - (missile_speed - MISSILE_MIN_SPEED) / (MISSILE_MAX_SPEED - MISSILE_MIN_SPEED);
                let possible_rotation = min_rotation + speed_multiplier * (PI * 0.8 - min_rotation);
                let angle = rng.gen_range(0.0, possible_rotation) - possible_rotation / 2.0;
                Rotation2::new(PI + angle) * missile.velocity.normalize() * PARTICLE_SPEED
            };

            entities
                .build_entity()
                .with(
                    SpellParticle {
                        inertia: missile.velocity,
                        velocity: particle_velocity,
                        frame_spawned: frame_number,
                    },
                    &mut spell_particles,
                )
                .with(transform, &mut transforms)
                .build();
        }

        for (spell_particle_entity, spell_particle, particle_transform) in
            (&entities, &spell_particles, &mut transforms).join()
        {
            if game_time_service.seconds_to_frame(spell_particle.frame_spawned) > 0.25 {
                entities
                    .delete(spell_particle_entity)
                    .expect("Expected to delete SpellParticle");
            } else {
                let displacement = (spell_particle.velocity + spell_particle.inertia)
                    * game_time_service.engine_time().fixed_seconds();
                *particle_transform.translation_mut() +=
                    Vector3::new(displacement.x, displacement.y, 0.0);
            }
        }
    }
}
