use amethyst::ecs::{Component, Entities, Entity, Join, ReadStorage, System, WriteExpect};

use std::iter::FromIterator;

use ha_core::ecs::{
    components::{missile::Missile, Monster, Player, WorldPosition},
    resources::world::{SavedWorldState, WorldStates},
    system_data::time::GameTimeService,
};
use std::collections::BTreeMap;

pub struct WorldStateSystem;

impl<'s> System<'s> for WorldStateSystem {
    type SystemData = (
        GameTimeService<'s>,
        WriteExpect<'s, WorldStates>,
        Entities<'s>,
        ReadStorage<'s, WorldPosition>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Missile>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            _world_states,
            entities,
            _world_positions,
            players,
            monsters,
            missiles,
        ): Self::SystemData,
    ) {
        let mut world_state = SavedWorldState {
            frame_number: game_time_service.game_frame_number(),
            ..SavedWorldState::default()
        };

        world_state
            .players
            .append(&mut copy_from_storage(&entities, &players));
        world_state
            .monsters
            .append(&mut copy_from_storage(&entities, &monsters));
        world_state
            .missiles
            .append(&mut copy_from_storage(&entities, &missiles));
    }
}

fn copy_from_storage<T: Clone + Component>(
    entities: &Entities,
    storage: &ReadStorage<T>,
) -> BTreeMap<Entity, T> {
    BTreeMap::from_iter(
        (entities, storage)
            .join()
            .map(|(entity, component)| (entity, component.clone())),
    )
}
