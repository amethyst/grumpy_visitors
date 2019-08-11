use amethyst::{
    core::Time,
    ecs::{Join, ReadExpect, System, WriteStorage},
};

use ha_core::{
    ecs::components::{Monster, WorldPosition},
    math::{Vector2, ZeroVector},
};

use crate::ecs::resources::MonsterDefinitions;

pub struct MonsterMovementSystem;

impl<'s> System<'s> for MonsterMovementSystem {
    type SystemData = (
        ReadExpect<'s, Time>,
        ReadExpect<'s, MonsterDefinitions>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, monster_definitions, mut monsters, mut world_positions): Self::SystemData,
    ) {
        for (monster, world_position) in (&mut monsters, &mut world_positions).join() {
            let monster_definition = monster_definitions.0.get(&monster.name).unwrap();

            let monster_position = &mut **world_position;
            let monster_speed = monster_definition.base_speed;
            let time = time.delta_seconds();
            let travel_distance_squared = monster_speed * monster_speed * time * time;

            let displacement = monster.destination - *monster_position;
            *monster_position = if displacement.norm_squared() - travel_distance_squared < 0.01 {
                monster.velocity = Vector2::zero();
                monster.destination
            } else {
                monster.velocity = displacement.normalize() * monster_speed * time;
                *monster_position + monster.velocity
            };
        }
    }
}
