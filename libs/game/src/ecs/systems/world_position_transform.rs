use amethyst::{
    core::Transform,
    ecs::{Join, ReadStorage, System, WriteStorage},
};

use crate::ecs::components::WorldPosition;

pub struct WorldPositionTransformSystem;

impl<'s> System<'s> for WorldPositionTransformSystem {
    type SystemData = (ReadStorage<'s, WorldPosition>, WriteStorage<'s, Transform>);

    fn run(&mut self, (world_positions, mut transforms): Self::SystemData) {
        for (world_position, transform) in (&world_positions, &mut transforms).join() {
            transform.set_translation_xyz(
                world_position.x,
                world_position.y,
                transform.translation().z,
            );
        }
    }
}
