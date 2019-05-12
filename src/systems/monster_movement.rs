use amethyst::{
    core::{Time, Transform},
    ecs::{Join, Read, ReadExpect, System, WriteStorage},
};

use crate::{
    components::{Monster, WorldPosition},
    data_resources::MonsterDefinitions,
    Vector3,
};

pub struct MonsterMovementSystem;

impl<'s> System<'s> for MonsterMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        ReadExpect<'s, MonsterDefinitions>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, monster_definitions, monsters, mut transforms, mut world_positions): Self::SystemData,
    ) {
        for (monster, transform, world_position) in
            (&monsters, &mut transforms, &mut world_positions).join()
        {
            let monster_definition = monster_definitions.0.get(&monster.name).unwrap();

            let monster_position = &mut world_position.position;
            let monster_speed = monster_definition.base_speed;
            let time = time.delta_real_seconds();
            let travel_distance_squared = monster_speed * monster_speed * time * time;

            let displacement = monster.destination - *monster_position;
            *monster_position = if displacement.norm_squared() - travel_distance_squared < 0.01 {
                monster.destination
            } else {
                *monster_position + displacement.normalize() * monster_speed * time
            };

            transform.set_translation(Vector3::new(
                world_position.position.x,
                world_position.position.y,
                0.0,
            ));
        }
    }
}
