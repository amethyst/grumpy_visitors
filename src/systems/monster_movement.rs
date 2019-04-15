use amethyst::{
    core::{Time, Transform},
    ecs::{Join, Read, System, WriteStorage},
};

use crate::components::{Monster, WorldPosition};
use crate::Vector3;

pub struct MonsterMovementSystem;

impl<'s> System<'s> for MonsterMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, monsters, mut transforms, mut world_positions): Self::SystemData,
    ) {
        for (monster, transform, world_position) in (&monsters, &mut transforms, &mut world_positions).join() {
            world_position.position += monster.velocity * time.delta_real_seconds();

            transform.set_translation(Vector3::new(
                world_position.position.x,
                world_position.position.y,
                0.0,
            ));
        }
    }
}
