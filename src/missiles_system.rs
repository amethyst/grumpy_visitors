use amethyst::{
    core::{
        math::{clamp, Rotation2},
        Time, Transform,
    },
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
};

use std::time::{Duration, Instant};

use crate::{
    components::{Missile, Player, WorldPosition},
    Vector3,
};

pub struct MissilesSystem;

const MS_PER_FRAME: f32 = 1000.0 / 60.0;

const MISSILE_LIFESPAN_SECS: u64 = 5;
const MISSILE_MAX_SPEED: f32 = 700.0;
const MISSILE_MIN_SPEED: f32 = 200.0;
const TIME_TO_ACCELERATE: f32 = 2000.0;
const MISSILE_ACCELERATION: f32 =
    (MISSILE_MAX_SPEED - MISSILE_MIN_SPEED) / TIME_TO_ACCELERATE * MS_PER_FRAME;
const TIME_TO_ROTATE: f32 = 1000.0;
const MAX_ROTATION: f32 = std::f32::consts::PI / TIME_TO_ROTATE * MS_PER_FRAME;

impl<'s> System<'s> for MissilesSystem {
    type SystemData = (
        Read<'s, Time>,
        Entities<'s>,
        WriteStorage<'s, Missile>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
        ReadStorage<'s, Player>,
    );

    fn run(
        &mut self,
        (time, entities, mut missiles, mut transforms, mut world_positions, players): Self::SystemData,
    ) {
        let now = Instant::now();
        let (player, player_position) = (&players, &world_positions).join().next().unwrap();
        let Player {
            velocity: player_velocity,
            radius: player_radius,
        } = player;
        let player_position = player_position.position;

        for (entity, mut missile, transform, missile_position) in (
            &entities,
            &mut missiles,
            &mut transforms,
            &mut world_positions,
        )
            .join()
        {
            if now > missile.time_spawned + Duration::from_secs(MISSILE_LIFESPAN_SECS) {
                entities.delete(entity).unwrap();
                continue;
            }

            let missile_position = &mut missile_position.position;
            if (*missile_position - player_position).norm_squared() < player_radius * player_radius
            {
                entities.delete(entity).unwrap();
                continue;
            }

            let direction = player_position + player_velocity - *missile_position;
            let needed_angle = Rotation2::rotation_between(&missile.velocity, &direction).angle();
            let angle = needed_angle.abs().min(MAX_ROTATION) * needed_angle.signum();
            let a = if needed_angle.abs() > angle.abs() {
                -MISSILE_ACCELERATION
            } else {
                MISSILE_ACCELERATION
            };
            let current_speed = missile.velocity.norm();
            let speed = clamp(current_speed + a, MISSILE_MIN_SPEED, MISSILE_MAX_SPEED);
            let new_direction = Rotation2::new(angle) * missile.velocity.normalize();

            missile.velocity = new_direction * speed;

            *missile_position += missile.velocity * time.delta_real_seconds();
            transform.set_translation(Vector3::new(missile_position.x, missile_position.y, 0.0));
        }
    }
}
