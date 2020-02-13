use amethyst::ecs::{Entities, Join, ReadStorage, System};

use gv_core::ecs::{
    components::{missile::Missile, Dead},
    system_data::time::GameTimeService,
};

use crate::ecs::system_data::GameStateHelper;

pub struct MissileDyingSystem;

pub const MISSILE_TTL_SECS: f32 = 0.35;

impl<'s> System<'s> for MissileDyingSystem {
    type SystemData = (
        GameStateHelper<'s>,
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Dead>,
        ReadStorage<'s, Missile>,
    );

    fn run(
        &mut self,
        (game_state_helper, game_time_service, entities, dead, missiles): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        for (missile_entity, dead, _) in (&entities, &dead, &missiles).join() {
            let to_be_deleted =
                game_time_service.seconds_to_frame(dead.dead_since_frame) > MISSILE_TTL_SECS;
            if to_be_deleted {
                entities
                    .delete(missile_entity)
                    .expect("Expected to delete a Missile");
            }
        }
    }
}
