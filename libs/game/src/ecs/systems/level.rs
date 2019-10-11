use amethyst::ecs::{System, WriteExpect};

use std::time::Duration;

use ha_core::{
    actions::monster_spawn::{SpawnAction, SpawnActions, SpawnType},
    ecs::{
        resources::{net::EntityNetMetadataStorage, world::FramedUpdates, GameLevelState},
        system_data::time::GameTimeService,
    },
    math::Vector2,
};

use crate::{
    ecs::system_data::GameStateHelper,
    utils::world::{random_spawn_position, spawning_side},
};

const SECS_PER_LEVEL: u64 = 30;
const MIN_BORDERLINE_INTERVAL_SECS: f32 = 30.0;
const MAX_BORDERLINE_INTERVAL_SECS: f32 = 5.0;

#[derive(Default)]
pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    type SystemData = (
        GameStateHelper<'s>,
        GameTimeService<'s>,
        WriteExpect<'s, GameLevelState>,
        WriteExpect<'s, FramedUpdates<SpawnActions>>,
        WriteExpect<'s, EntityNetMetadataStorage>,
    );

    fn run(
        &mut self,
        (
            game_state_helper,
            game_time_service,
            mut game_level_state,
            mut spawn_actions,
            mut entity_net_metadata_storage,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() || !game_state_helper.is_authoritative() {
            return;
        }
        spawn_actions.reserve_updates(game_time_service.game_frame_number());
        let spawn_actions = spawn_actions
            .update_frame(game_time_service.game_frame_number())
            .unwrap_or_else(|| {
                panic!(
                    "Expected SpawnActions for frame {}",
                    game_time_service.game_frame_number()
                )
            });

        let now = game_time_service.level_duration();

        if now - game_level_state.spawn_level_started > Duration::from_secs(SECS_PER_LEVEL) {
            game_level_state.spawn_level += 1;
            game_level_state.spawn_level_started = now;
        }

        if game_time_service.game_frame_number() == 10 {
            spawn_actions.spawn_actions.push(SpawnAction {
                spawn_type: SpawnType::Single {
                    entity_net_id: Some(entity_net_metadata_storage.reserve_ids(1).start),
                    position: Vector2::new(0.0, 300.0),
                },
            });
        }

        let borderline_spawn_interval = MIN_BORDERLINE_INTERVAL_SECS
            - (game_level_state.spawn_level as f32 / 7.0).atan() / std::f32::consts::PI
                * 2.0
                * (MAX_BORDERLINE_INTERVAL_SECS - MIN_BORDERLINE_INTERVAL_SECS);
        let borderline_spawn_interval =
            Duration::from_millis((borderline_spawn_interval * 1000.0).round() as u64);
        if now - game_level_state.last_borderline_spawn > borderline_spawn_interval {
            game_level_state.last_borderline_spawn = now;

            let side = rand::random();

            let spawn_margin = 50.0;
            let (side_start, side_end, _) = spawning_side(side, &game_level_state);
            let d = (side_start - side_end) / spawn_margin;
            let monsters_to_spawn = num::Float::max(d.x.abs(), d.y.abs()).round() as usize;

            let entity_net_id_range = if game_state_helper.is_multiplayer() {
                Some(entity_net_metadata_storage.reserve_ids(monsters_to_spawn))
            } else {
                None
            };

            log::trace!(
                "Spawning {} monster(s) (SpawnType::Borderline)",
                monsters_to_spawn
            );
            spawn_actions.spawn_actions.push(SpawnAction {
                spawn_type: SpawnType::Borderline {
                    count: monsters_to_spawn as u8,
                    entity_net_id_range,
                    side,
                },
            });
        }

        let random_spawn_interval = Duration::from_secs(1);
        let monsters_to_spawn = game_level_state.spawn_level.min(255) as u8;
        if now - game_level_state.last_random_spawn > random_spawn_interval {
            game_level_state.last_random_spawn = now;
            log::trace!(
                "Spawning {} monster(s) (SpawnType::Single)",
                monsters_to_spawn
            );
            for _ in 0..monsters_to_spawn {
                spawn_actions.spawn_actions.push(SpawnAction {
                    spawn_type: SpawnType::Single {
                        entity_net_id: Some(entity_net_metadata_storage.reserve_ids(1).start),
                        position: random_spawn_position(&game_level_state),
                    },
                });
            }
        }
    }
}
