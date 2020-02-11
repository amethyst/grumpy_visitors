use amethyst::ecs::{Entities, Join, ReadStorage, System};

use gv_animation_prefabs::{AnimationId, MONSTER_BODY};
use gv_core::ecs::{
    components::{Dead, Monster},
    system_data::time::GameTimeService,
};

use crate::ecs::{system_data::GameStateHelper, systems::AnimationsSystemData};

pub struct MonsterDyingSystem;

impl<'s> System<'s> for MonsterDyingSystem {
    type SystemData = (
        GameStateHelper<'s>,
        GameTimeService<'s>,
        AnimationsSystemData<'s>,
        Entities<'s>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Dead>,
    );

    fn run(
        &mut self,
        (
            game_state_helper,
            game_time_service,
            mut animations_system_data,
            entities,
            monsters,
            dead,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        // TODO: move to animation system?
        for (monster_entity, dead, _) in (&entities, &dead, &monsters).join() {
            if game_time_service.game_frame_number() == dead.frame_acknowledged {
                animations_system_data.remove_animation(
                    monster_entity,
                    MONSTER_BODY,
                    AnimationId::Walk,
                );
                animations_system_data.play_animation(
                    monster_entity,
                    MONSTER_BODY,
                    AnimationId::Death,
                );
            }
        }
    }
}
