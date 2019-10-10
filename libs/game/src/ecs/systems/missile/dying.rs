use amethyst::{
    core::HiddenPropagate,
    ecs::{Entities, Join, ReadStorage, System},
};

use ha_core::ecs::{
    components::{missile::Missile, Dead},
    resources::world::LAG_COMPENSATION_FRAMES_LIMIT,
    system_data::time::GameTimeService,
};

use crate::ecs::system_data::GameStateHelper;

pub struct MissileDyingSystem;

const DYING_TIME_FRAMES: u64 = LAG_COMPENSATION_FRAMES_LIMIT as u64 * 3;

impl<'s> System<'s> for MissileDyingSystem {
    type SystemData = (
        GameStateHelper<'s>,
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Dead>,
        ReadStorage<'s, HiddenPropagate>,
        ReadStorage<'s, Missile>,
    );

    fn run(
        &mut self,
        (game_state_helper, game_time_service, entities, dead, hidden_propagates, missiles): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        for (missile_entity, dead, _, _) in (&entities, &dead, &hidden_propagates, &missiles).join()
        {
            let to_be_deleted = !game_state_helper.is_multiplayer()
                || game_time_service.game_frame_number() - dead.dead_since_frame
                    > DYING_TIME_FRAMES;
            if to_be_deleted {
                entities
                    .delete(missile_entity)
                    .expect("Expected to delete a Missile");
            }
        }
    }
}
