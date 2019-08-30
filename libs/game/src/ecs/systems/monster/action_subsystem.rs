#[cfg(not(feature = "client"))]
use amethyst::ecs::Join;
use amethyst::ecs::{Entities, Entity, ReadExpect};

use ha_core::{
    actions::{
        mob::{MobAction, MobAttackAction, MobAttackType},
        Action,
    },
    ecs::{
        components::{damage_history::DamageHistory, Monster, Player, WorldPosition},
        resources::GameLevelState,
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
};

use crate::ecs::{resources::MonsterDefinitions, systems::WriteStorageCell};
#[cfg(not(feature = "client"))]
use crate::utils::world::random_scene_position;

#[cfg(not(feature = "client"))]
const MAX_IDLE_TIME_SECS: f32 = 0.5;

pub struct MonsterActionSubsystem<'s> {
    pub entities: &'s Entities<'s>,
    pub game_time_service: &'s GameTimeService<'s>,
    pub monster_definitions: &'s ReadExpect<'s, MonsterDefinitions>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub players: WriteStorageCell<'s, Player>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
    pub damage_histories: WriteStorageCell<'s, DamageHistory>,
    pub monsters: WriteStorageCell<'s, Monster>,
}

impl<'s> MonsterActionSubsystem<'s> {
    pub fn decide_monster_action(
        &self,
        entity: Entity,
        updated_position: &Option<WorldPosition>,
        action: &Option<MobAction<Entity>>,
        frame_number: u64,
    ) {
        let mut monsters = self.monsters.borrow_mut();
        let monster = monsters.get_mut(entity).expect("Expected a Monster");

        let monster_position = if let Some(updated_position) = updated_position {
            let mut world_positions = self.world_positions.borrow_mut();
            let monster_position = world_positions
                .get_mut(entity)
                .expect("Expected a WorldPosition");
            *monster_position = updated_position.clone();
            monster_position.clone()
        } else {
            self.world_positions
                .borrow()
                .get(entity)
                .expect("Expected a WorldPosition")
                .clone()
        };

        let world_positions = self.world_positions.borrow();
        let new_action = action
            .clone()
            .or_else(|| self.new_action(&monster, monster_position.clone()));

        let new_destination = if let Some(ref new_action) = new_action {
            match new_action {
                MobAction::Move(position) => Some(*position),
                MobAction::Chase(entity) => Some(**world_positions.get(*entity).unwrap()),
                MobAction::Attack(MobAttackAction {
                    target,
                    attack_type,
                }) => match attack_type {
                    MobAttackType::Melee => Some(**world_positions.get(*target).unwrap()),
                    _ => Some(*monster_position),
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
                frame_number,
                action: Some(action),
            }
        }
    }

    pub fn process_monster_movement(&self, entity: Entity) {
        let mut monsters = self.monsters.borrow_mut();
        let mut world_positions = self.world_positions.borrow_mut();
        let monster = monsters.get_mut(entity).expect("Expected a Monster");
        let monster_position = world_positions
            .get_mut(entity)
            .expect("Expected a WorldPosition");

        let monster_definition = self
            .monster_definitions
            .0
            .get(&monster.name)
            .expect("Expected a MonsterDefinition");

        let monster_position = &mut **monster_position;
        let monster_speed = monster_definition.base_speed;
        let time = self.game_time_service.engine_time().fixed_seconds();
        let travel_distance_squared = monster_speed * monster_speed * time * time;

        let displacement = monster.destination - *monster_position;
        *monster_position = if displacement.norm_squared() - travel_distance_squared < 0.01 {
            monster.velocity = Vector2::zero();
            monster.destination
        } else {
            monster.velocity = displacement.normalize() * monster_speed * time;
            *monster_position + monster.velocity
        };
    }

    #[cfg(not(feature = "client"))]
    fn new_action(
        &self,
        monster: &Monster,
        monster_position: WorldPosition,
    ) -> Option<MobAction<Entity>> {
        let players = self.players.borrow();
        let world_positions = self.world_positions.borrow();

        let monster_definition = self
            .monster_definitions
            .0
            .get(&monster.name)
            .expect("Expected a MonsterDefinition");

        match monster.action.action {
            Some(MobAction::Idle) | None => {
                if let Some((entity, _player_position)) = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
                    200.0,
                ) {
                    Some(MobAction::Chase(entity))
                } else {
                    let time_being_idle = self
                        .game_time_service
                        .seconds_to_frame(monster.action.frame_number);
                    if MAX_IDLE_TIME_SECS < time_being_idle {
                        Some(MobAction::Move(random_scene_position(
                            &*self.game_level_state,
                        )))
                    } else {
                        None
                    }
                }
            }
            Some(MobAction::Move(destination)) => {
                if let Some((entity, _player_position)) = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
                    200.0,
                ) {
                    Some(MobAction::Chase(entity))
                } else if (*monster_position - destination).norm_squared() < 0.01 {
                    Some(MobAction::Idle)
                } else {
                    None
                }
            }
            Some(MobAction::Chase(_)) => {
                if let Some((target, _player_position)) = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
                    monster.radius,
                ) {
                    // TODO: synchronize damage histories.

                    //                let damage_history = damage_histories
                    //                    .get_mut(target)
                    //                    .expect("Expected player's DamageHistory");
                    //                damage_history.add_entry(
                    //                    game_time_service.game_frame_number(),
                    //                    DamageHistoryEntry {
                    //                        damage: monster.attack_damage,
                    //                    },
                    //                );
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
                        self.game_time_service
                            .seconds_to_frame(monster.action.frame_number)
                            < cooldown
                    }
                    _ => false,
                };
                let player_in_radius = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
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
        }
    }

    #[cfg(feature = "client")]
    fn new_action(
        &self,
        _monster: &Monster,
        _monster_position: WorldPosition,
    ) -> Option<MobAction<Entity>> {
        None
    }
}

#[cfg(not(feature = "client"))]
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
