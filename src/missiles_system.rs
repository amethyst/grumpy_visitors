use amethyst::{
    core::{Time, Transform},
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
};

use std::time::{Instant, Duration};

use crate::{
    components::{Missile, WorldPosition, Player},
    Vector3,
};

pub struct MissilesSystem;

const MISSILE_LIFESPAN_SECS: u64 = 5;
const MISSILE_ACCELERATION: f32 = 30.0;

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
        let Player { velocity: player_velocity, radius: player_radius } = player.clone();
        let player_position = player_position.position.clone();

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
            if (*missile_position - player_position).norm_squared() < player_radius * player_radius {
                entities.delete(entity).unwrap();
                continue;
            }

            let direction = {
                let direction_to_player = player_position - *missile_position;
                let d = direction_to_player.normalize().norm();
                let a = MISSILE_ACCELERATION;

                let velocity_d = player_velocity - missile.velocity;

                let v = (-velocity_d).dot(&direction_to_player) ;

                let toi = -v / a + (v * v / (a * a) + 2.0 * d / a).sqrt();

                let impact_position = player_position + player_velocity * toi;

                impact_position - *missile_position
            };

            missile.velocity = direction.normalize() * (missile.velocity.norm() + MISSILE_ACCELERATION);

            *missile_position += missile.velocity * time.delta_real_seconds();
            transform.set_position(Vector3::new(
                missile_position.x,
                missile_position.y,
                0.0,
            ));
        }
    }
}
