use amethyst::ecs::{Entities, Join, ReadStorage, System, WriteStorage};

use std::time::Duration;

use ha_core::ecs::{
    components::{missile::MissileTarget, Dead, Monster, PlayerActions, WorldPosition},
    system_data::time::GameTimeService,
};

use crate::{
    ecs::{factories::MissileFactory, systems::missile::physics::MISSILE_MAX_SPEED},
    utils::world::closest_monster,
};

pub struct MissileSpawnerSystem;

const SPELL_CAST_COOLDOWN: Duration = Duration::from_millis(500);

impl<'s> System<'s> for MissileSpawnerSystem {
    type SystemData = (
        GameTimeService<'s>,
        Entities<'s>,
        MissileFactory<'s>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Dead>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, PlayerActions>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            entities,
            mut missile_factory,
            monsters,
            dead,
            mut world_positions,
            mut player_actions,
        ): Self::SystemData,
    ) {
        let now = game_time_service.level_duration();

        for (player_actions, _) in (&mut player_actions, !&dead).join() {
            if let Some(cast_action) = player_actions.cast_action.action.as_ref() {
                if player_actions.cast_action.frame_number < game_time_service.game_frame_number() {
                    continue;
                }
                if player_actions.last_spell_cast + SPELL_CAST_COOLDOWN > now {
                    continue;
                }
                player_actions.last_spell_cast = now;

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
