use amethyst::{
    core::{
        math::{clamp, Rotation2},
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{Entities, Join, ReadExpect, System, SystemData, World, WriteStorage},
};

use std::time::Duration;

use ha_core::ecs::{
    components::{
        damage_history::{DamageHistory, DamageHistoryEntry},
        missile::{Missile, MissileTarget},
        Monster, WorldPosition,
    },
    resources::GameLevelState,
    system_data::{game_state_helper::GameStateHelper, time::GameTimeService},
};

use crate::utils::world::{closest_monster, find_first_hit_monster, random_scene_position};

pub const MISSILE_MAX_SPEED: f32 = 300.0;

const MS_PER_FRAME: f32 = 1000.0 / 60.0;

const MISSILE_LIFESPAN_SECS: u64 = 5;
const MISSILE_MIN_SPEED: f32 = 80.0;
const TIME_TO_ACCELERATE: f32 = 2000.0;
const MISSILE_ACCELERATION: f32 =
    (MISSILE_MAX_SPEED - MISSILE_MIN_SPEED) / TIME_TO_ACCELERATE * MS_PER_FRAME;
const TIME_TO_ROTATE: f32 = 1000.0;
const MAX_ROTATION: f32 = std::f32::consts::PI / TIME_TO_ROTATE * MS_PER_FRAME;

#[derive(SystemDesc)]
pub struct MissileSystem;

impl<'s> System<'s> for MissileSystem {
    type SystemData = (
        GameTimeService<'s>,
        GameStateHelper<'s>,
        ReadExpect<'s, GameLevelState>,
        Entities<'s>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, Missile>,
        WriteStorage<'s, DamageHistory>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_state_helper,
            game_scene,
            entities,
            monsters,
            mut missiles,
            mut damage_histories,
            mut world_positions,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        let now = game_time_service.level_duration();

        for (missile_entity, mut missile) in (&entities, &mut missiles).join() {
            let missile_position = **world_positions
                .get(missile_entity)
                .expect("Expected a missile");
            if now > missile.time_spawned + Duration::from_secs(MISSILE_LIFESPAN_SECS) {
                entities.delete(missile_entity).unwrap();
                continue;
            }

            let (destination, new_target) = match missile.target {
                MissileTarget::Target(target) => {
                    if let Some(target_position) = world_positions.get(target) {
                        (**target_position, None)
                    } else if let Some((target, target_position)) =
                        closest_monster(missile_position, &world_positions, &entities, &monsters)
                    {
                        (target_position, Some(MissileTarget::Target(target)))
                    } else {
                        let target_position = random_scene_position(&*game_scene);
                        (
                            target_position,
                            Some(MissileTarget::Destination(target_position)),
                        )
                    }
                }
                MissileTarget::Destination(destination) => {
                    if let Some((target, target_position)) =
                        closest_monster(missile_position, &world_positions, &entities, &monsters)
                    {
                        (target_position, Some(MissileTarget::Target(target)))
                    } else if (destination - missile_position).norm_squared()
                        > missile.velocity.norm_squared()
                    {
                        (destination, None)
                    } else {
                        let target_position = random_scene_position(&*game_scene);
                        (
                            target_position,
                            Some(MissileTarget::Destination(target_position)),
                        )
                    }
                }
            };
            if let Some(new_target) = new_target {
                missile.target = new_target;
            }

            let direction = if let MissileTarget::Target(target) = missile.target {
                if let Some(hit_monster) = find_first_hit_monster(
                    missile_position,
                    missile.radius,
                    &monsters,
                    &world_positions,
                    &entities,
                ) {
                    damage_histories
                        .get_mut(hit_monster)
                        .expect("Expected a DamageHistory")
                        .add_entry(
                            game_time_service.game_frame_number(),
                            DamageHistoryEntry {
                                damage: missile.damage,
                            },
                        );
                    entities.delete(missile_entity).unwrap();
                    continue;
                }
                let monster = monsters.get(target).expect("Expected a targeted Monster");
                destination + monster.velocity - missile_position
            } else {
                destination
            };
            let needed_angle = Rotation2::rotation_between(&missile.velocity, &direction).angle();
            let angle = needed_angle.abs().min(MAX_ROTATION) * needed_angle.signum();
            let a = if needed_angle.abs() > angle.abs() {
                -MISSILE_ACCELERATION
            } else {
                MISSILE_ACCELERATION
            };
            let current_speed = missile.velocity.norm();
            let speed = clamp(current_speed + a, MISSILE_MIN_SPEED, MISSILE_MAX_SPEED);
            let new_direction = Rotation2::new(angle) * missile.velocity.normalize();

            missile.velocity = new_direction * speed;

            let missile_position = world_positions
                .get_mut(missile_entity)
                .expect("Expected a Missile");
            **missile_position +=
                missile.velocity * game_time_service.engine_time().fixed_seconds();
        }
    }
}
