use amethyst::{
    core::math::{clamp, Rotation2},
    ecs::{Entities, Join, ReadExpect},
};
use gv_core::profile_scope;

use gv_core::ecs::{
    components::{
        damage_history::{DamageHistory, DamageHistoryEntry},
        missile::{Missile, MissileTarget},
        Dead, Monster, WorldPosition,
    },
    resources::GameLevelState,
    system_data::time::GameTimeService,
};

use crate::{
    ecs::{system_data::GameStateHelper, systems::WriteStorageCell},
    utils::{
        entities::{is_dead, missile_energy},
        world::{closest_monster, find_first_hit_monster, random_scene_position},
    },
};

pub const MISSILE_MAX_SPEED: f32 = 300.0;
pub const MISSILE_MIN_SPEED: f32 = 80.0;
pub const MISSILE_TIME_TO_FADE: f32 = 0.5;
pub const MISSILE_LIFESPAN_SECS: f32 = 5.0;

const MS_PER_FRAME: f32 = 1000.0 / 60.0;

const TIME_TO_ACCELERATE: f32 = 2000.0;
const MISSILE_ACCELERATION: f32 =
    (MISSILE_MAX_SPEED - MISSILE_MIN_SPEED) / TIME_TO_ACCELERATE * MS_PER_FRAME;
const TIME_TO_ROTATE: f32 = 1000.0;
const MAX_ROTATION: f32 = std::f32::consts::PI / TIME_TO_ROTATE * MS_PER_FRAME;

pub struct MissilePhysicsSubsystem<'s> {
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_state_helper: &'s GameStateHelper<'s>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub entities: &'s Entities<'s>,
    pub monsters: WriteStorageCell<'s, Monster>,
    pub missiles: WriteStorageCell<'s, Missile>,
    pub dead: WriteStorageCell<'s, Dead>,
    pub damage_histories: WriteStorageCell<'s, DamageHistory>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
}

impl<'s> MissilePhysicsSubsystem<'s> {
    pub fn process_physics(&self, frame_number: u64) {
        profile_scope!("MissilePhysicsSubsystem::process_physics");
        let monsters = self.monsters.borrow();
        let mut missiles = self.missiles.borrow_mut();
        let mut dead = self.dead.borrow_mut();
        let mut damage_histories = self.damage_histories.borrow_mut();
        let mut world_positions = self.world_positions.borrow_mut();

        for (missile_entity, mut missile) in (self.entities, &mut *missiles).join() {
            let is_dead = is_dead(missile_entity, &*dead, frame_number);
            if missile.frame_spawned > frame_number || is_dead {
                continue;
            }

            let missile_energy =
                missile_energy(&missile, is_dead, &self.game_time_service, frame_number);
            if missile_energy == 0.0 {
                let dead_since_frame = frame_number + 1;
                let frame_acknowledged =
                    dead_since_frame.max(self.game_time_service.game_frame_number());
                dead.insert(
                    missile_entity,
                    Dead::new(dead_since_frame, frame_acknowledged),
                )
                .expect("Expected to insert a Dead component");
                continue;
            }

            let missile_position = **world_positions
                .get(missile_entity)
                .expect("Expected a missile");

            let (destination, new_target) = match missile.target {
                MissileTarget::Target(target) => {
                    if let Some(target_position) = world_positions.get(target) {
                        (**target_position, None)
                    } else if let Some((target, target_position)) = closest_monster(
                        missile_position,
                        &world_positions,
                        &self.entities,
                        &monsters,
                        &*dead,
                        frame_number,
                    ) {
                        (target_position, Some(MissileTarget::Target(target)))
                    } else {
                        let target_position = random_scene_position(self.game_level_state);
                        (
                            target_position,
                            Some(MissileTarget::Destination(target_position)),
                        )
                    }
                }
                MissileTarget::Destination(destination) => {
                    if let Some((target, target_position)) = closest_monster(
                        missile_position,
                        &world_positions,
                        &self.entities,
                        &monsters,
                        &*dead,
                        frame_number,
                    ) {
                        (target_position, Some(MissileTarget::Target(target)))
                    } else if (destination - missile_position).norm_squared()
                        > missile.velocity.norm_squared()
                    {
                        (destination, None)
                    } else {
                        let target_position = random_scene_position(&*self.game_level_state);
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
                if missile_energy >= 1.0 {
                    if let Some(hit_monster) = find_first_hit_monster(
                        missile_position,
                        missile.radius,
                        &monsters,
                        &world_positions,
                        &self.entities,
                        &*dead,
                        frame_number,
                    ) {
                        if self.game_state_helper.is_authoritative() {
                            damage_histories
                                .get_mut(hit_monster)
                                .expect("Expected a DamageHistory")
                                .add_entry(
                                    frame_number,
                                    DamageHistoryEntry {
                                        damage: missile.damage,
                                    },
                                );
                        }
                        let dead_since_frame = frame_number + 1;
                        let frame_acknowledged =
                            dead_since_frame.max(self.game_time_service.game_frame_number());
                        dead.insert(
                            missile_entity,
                            Dead::new(dead_since_frame, frame_acknowledged),
                        )
                        .expect("Expected to insert a Dead component");
                        continue;
                    }
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
                missile.velocity * self.game_time_service.engine_time().fixed_seconds();
        }
    }
}
