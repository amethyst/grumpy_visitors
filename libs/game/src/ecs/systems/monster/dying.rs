use amethyst::{
    core::HiddenPropagate,
    ecs::{Entities, Join, ReadStorage, System},
};

use gv_core::ecs::{
    components::{Dead, Monster},
    resources::world::SAVED_WORLD_STATES_LIMIT,
    system_data::time::GameTimeService,
};

use crate::ecs::system_data::GameStateHelper;

pub struct MonsterDyingSystem;

// Anything more clever?
const DYING_TIME_FRAMES: u64 = SAVED_WORLD_STATES_LIMIT as u64;

impl<'s> System<'s> for MonsterDyingSystem {
    type SystemData = (
        GameStateHelper<'s>,
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Dead>,
        ReadStorage<'s, HiddenPropagate>,
    );

    fn run(
        &mut self,
        (
            game_state_helper,
            game_time_service,
            entities,
            monsters,
            dead,
            hidden_propagates,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        for (monster_entity, dead, _, _) in (&entities, &dead, &hidden_propagates, &monsters).join()
        {
            let to_be_deleted = !game_state_helper.is_multiplayer()
                || game_time_service
                    .game_frame_number()
                    .saturating_sub(dead.dead_since_frame)
                    > DYING_TIME_FRAMES;
            if to_be_deleted {
                entities
                    .delete(monster_entity)
                    .expect("Expected to delete a Monster");
            }
        }
    }
}
