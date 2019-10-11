use amethyst::ecs::{storage::GenericReadStorage, Entity};

use ha_core::ecs::components::Dead;

pub fn is_dead(
    entity: Entity,
    dead: &impl GenericReadStorage<Component = Dead>,
    frame_number: u64,
) -> bool {
    dead.get(entity)
        .map_or(false, |dead| dead.dead_since_frame <= frame_number)
}
