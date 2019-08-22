use amethyst::ecs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use ha_core::{
    actions::{
        mob::{MobAction, MobAttackAction, MobAttackType},
        Action,
    },
    ecs::{
        components::{
            damage_history::{DamageHistory, DamageHistoryEntry},
            Monster, Player, WorldPosition,
        },
        resources::GameLevelState,
        system_data::time::GameTimeService,
    },
    math::Vector2,
};

use crate::{ecs::resources::MonsterDefinitions, utils::world::random_scene_position};

const MAX_IDLE_TIME_SECS: f32 = 0.5;

pub struct MonsterActionSystem;

impl<'s> System<'s> for MonsterActionSystem {
    type SystemData = (
        Entities<'s>,
        GameTimeService<'s>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadExpect<'s, GameLevelState>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, DamageHistory>,
        WriteStorage<'s, Monster>,
    );

    fn run(
        &mut self,
        (
            entities,
            game_time_service,
            monster_definitions,
            game_scene,
            players,
            world_positions,
            mut damage_histories,
            mut monsters,
        ): Self::SystemData,
    ) {
        for (mut monster, monster_position) in (&mut monsters, &world_positions).join() {
            let monster_definition = monster_definitions
                .0
                .get(&monster.name)
                .expect("Expected a monster definition");

            let new_action = match monster.action.action {
                Some(MobAction::Idle) | None => {
                    if let Some((entity, _player_position)) = find_player_in_radius(
                        (&entities, &players, &world_positions).join(),
                        **monster_position,
                        200.0,
                    ) {
                        Some(MobAction::Chase(entity))
                    } else {
                        let time_being_idle =
                            game_time_service.seconds_to_frame(monster.action.frame_number);
                        if MAX_IDLE_TIME_SECS < time_being_idle {
                            Some(MobAction::Move(random_scene_position(&*game_scene)))
                        } else {
                            None
                        }
                    }
                }
                Some(MobAction::Move(destination)) => {
                    if let Some((entity, _player_position)) = find_player_in_radius(
                        (&entities, &players, &world_positions).join(),
                        **monster_position,
                        200.0,
                    ) {
                        Some(MobAction::Chase(entity))
                    } else if (**monster_position - destination).norm_squared() < 0.01 {
                        Some(MobAction::Idle)
                    } else {
                        None
                    }
                }
                Some(MobAction::Chase(_)) => {
                    if let Some((target, _player_position)) = find_player_in_radius(
                        (&entities, &players, &world_positions).join(),
                        **monster_position,
                        monster.radius,
                    ) {
                        let damage_history = damage_histories
                            .get_mut(target)
                            .expect("Expected player's DamageHistory");
                        damage_history.add_entry(
                            game_time_service.game_frame_number(),
                            DamageHistoryEntry {
                                damage: monster.attack_damage,
                            },
                        );
                        Some(MobAction::Attack(MobAttackAction {
                            target,
                            attack_type: monster_definition.attack_type.randomize_params(0.2),
                        }))
                    } else {
                        None
                    }
                }
                Some(MobAction::Attack(ref attack_action)) => {
                    let is_cooling_down = match attack_action.attack_type {
                        MobAttackType::SlowMelee { cooldown } => {
                            game_time_service.seconds_to_frame(monster.action.frame_number)
                                < cooldown
                        }
                        _ => false,
                    };
                    let player_in_radius = find_player_in_radius(
                        (&entities, &players, &world_positions).join(),
                        **monster_position,
                        monster.radius,
                    );
                    match (&attack_action.attack_type, player_in_radius) {
                        // TODO: implement cooling down for other attacks as well.
                        (MobAttackType::SlowMelee { .. }, _) if is_cooling_down => None,
                        (_, Some((target, _player_position))) => {
                            Some(MobAction::Attack(MobAttackAction {
                                target,
                                attack_type: monster_definition.attack_type.randomize_params(0.2),
                            }))
                        }
                        (_, None) => Some(MobAction::Idle),
                    }
                }
            };

            let new_destination = if let Some(ref new_action) = new_action {
                match new_action {
                    MobAction::Move(position) => Some(*position),
                    MobAction::Chase(entity) => Some(**world_positions.get(*entity).unwrap()),
                    MobAction::Attack(MobAttackAction {
                        target,
                        attack_type,
                    }) => match attack_type {
                        MobAttackType::Melee => Some(**world_positions.get(*target).unwrap()),
                        _ => Some(**monster_position),
                    },
                    _ => None,
                }
            } else {
                match monster.action.action {
                    Some(MobAction::Chase(entity)) => Some(**world_positions.get(entity).unwrap()),
                    _ => None,
                }
            };

            if let Some(destination) = new_destination {
                monster.destination = destination;
            }

            if let Some(action) = new_action {
                monster.action = Action {
                    frame_number: game_time_service.game_frame_number(),
                    action: Some(action),
                }
            }
        }
    }
}

fn find_player_in_radius<'a>(
    mut players: impl Iterator<Item = (Entity, &'a Player, &'a WorldPosition)>,
    position: Vector2,
    radius: f32,
) -> Option<(Entity, &'a WorldPosition)> {
    let radius_squared = radius * radius;
    players
        .find(|(_, player, player_position)| {
            let player_radius_squared = player.radius * player.radius;
            (position - ***player_position).norm_squared() < radius_squared + player_radius_squared
        })
        .map(|(entity, _, player_position)| (entity, player_position))
}
