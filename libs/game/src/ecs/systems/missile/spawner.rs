use amethyst::ecs::{Entities, Join, ReadStorage, System, WriteStorage};

use std::time::Duration;

use ha_core::ecs::{
    components::{
        missile::MissileTarget, Dead, Monster, PlayerActions, PlayerLastCastedSpells, WorldPosition,
    },
    system_data::time::GameTimeService,
};

use crate::{
    ecs::{
        factories::MissileFactory, system_data::GameStateHelper,
        systems::missile::physics::MISSILE_MAX_SPEED,
    },
    utils::world::closest_monster,
};

pub struct MissileSpawnerSystem;

const SPELL_CAST_COOLDOWN: Duration = Duration::from_millis(500);

impl<'s> System<'s> for MissileSpawnerSystem {
    type SystemData = (
        GameTimeService<'s>,
        GameStateHelper<'s>,
        Entities<'s>,
        MissileFactory<'s>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Dead>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, PlayerActions>,
        WriteStorage<'s, PlayerLastCastedSpells>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_state_helper,
            entities,
            mut missile_factory,
            monsters,
            dead,
            mut world_positions,
            mut player_actions,
            mut player_last_casted_spells,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        let now = game_time_service.level_duration();

        for (player_actions, player_last_casted_spells, _) in
            (&mut player_actions, &mut player_last_casted_spells, !&dead).join()
        {
            if let Some(cast_action) = player_actions.cast_action.as_ref() {
                if player_last_casted_spells.missile + SPELL_CAST_COOLDOWN > now {
                    continue;
                }
                player_last_casted_spells.missile = now;

                let search_result = closest_monster(
                    cast_action.target_position,
                    &world_positions,
                    &entities,
                    &monsters,
                );

                let target = if let Some((monster, _)) = search_result {
                    MissileTarget::Target(monster)
                } else {
                    MissileTarget::Destination(cast_action.target_position)
                };
                let direction = cast_action.target_position - cast_action.cast_position;
                let velocity = direction.normalize() * MISSILE_MAX_SPEED;

                missile_factory.create(
                    &mut world_positions,
                    5.0,
                    target,
                    velocity,
                    now,
                    cast_action.cast_position,
                );
            }
        }
    }
}
