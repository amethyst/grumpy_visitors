use amethyst::ecs::{Entities, Entity, Join, ReadExpect, ReadStorage, WriteStorage};

use ha_core::{
    actions::{
        mob::{MobAction, MobAttackAction, MobAttackType},
        Action,
    },
    ecs::{
        components::{
            damage_history::{DamageHistory, DamageHistoryEntry},
            ClientPlayerActions, EntityNetMetadata, Monster, NetWorldPosition, Player,
            WorldPosition,
        },
        resources::GameLevelState,
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
    net::{NetIdentifier, NetUpdateWithPosition},
};

use crate::{
    ecs::{
        resources::MonsterDefinitions,
        system_data::GameStateHelper,
        systems::{OutcomingNetUpdates, WriteStorageCell},
    },
    utils::world::random_scene_position,
};

const MAX_IDLE_TIME_SECS: f32 = 0.5;

pub struct MonsterActionSubsystem<'s> {
    pub entities: &'s Entities<'s>,
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_state_helper: &'s GameStateHelper<'s>,
    pub monster_definitions: &'s ReadExpect<'s, MonsterDefinitions>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub client_player_actions: &'s ReadStorage<'s, ClientPlayerActions>,
    pub entity_net_metadata: WriteStorageCell<'s, EntityNetMetadata>,
    pub players: WriteStorageCell<'s, Player>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
    pub net_world_positions: WriteStorageCell<'s, NetWorldPosition>,
    pub damage_histories: WriteStorageCell<'s, DamageHistory>,
}

pub struct ApplyMonsterActionNetArgs<'a> {
    pub entity_net_id: NetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    /// Only clients receive monster action updates.
    pub updates: Option<(WorldPosition, MobAction<Entity>)>,
}

impl<'s> MonsterActionSubsystem<'s> {
    pub fn decide_monster_action<'a>(
        &self,
        frame_number: u64,
        entity: Entity,
        monster: &mut Monster,
        net_args: Option<ApplyMonsterActionNetArgs<'a>>,
    ) {
        let updated_position = net_args
            .as_ref()
            .and_then(|net_args| net_args.updates.as_ref().map(|update| update.0.clone()));
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

        let new_action = if self.game_state_helper.is_multiplayer() {
            let ApplyMonsterActionNetArgs {
                entity_net_id,
                outcoming_net_updates,
                updates,
            } = net_args.expect("Expected ApplyMonsterActionNetArgs in multiplayer");

            if self.game_state_helper.is_authoritative() {
                let action = self.new_action(frame_number, &monster, monster_position.clone());
                if let Some(action) = &action {
                    let update = NetUpdateWithPosition {
                        entity_net_id,
                        position: monster_position.clone(),
                        data: action.load_entity_net_id(&*self.entity_net_metadata.borrow_mut()),
                    };
                    add_mob_action_update(outcoming_net_updates, update)
                }
                action
            } else {
                updates.map(|updates| updates.1)
            }
        } else {
            self.new_action(frame_number, &monster, monster_position.clone())
        };

        let world_positions = self.world_positions.borrow();
        let net_world_positions = self.net_world_positions.borrow();
        let is_multiplayer = self.game_state_helper.is_multiplayer();
        let new_destination = if let Some(ref new_action) = new_action {
            log::trace!(
                "Applying a new mob ({}) action for frame {} (current frame {}): {:?}",
                entity.id(),
                frame_number,
                self.game_time_service.game_frame_number(),
                new_action
            );
            match new_action {
                MobAction::Move(position) => Some(*position),
                MobAction::Chase(target) => Some(target_position(
                    *target,
                    &world_positions,
                    &net_world_positions,
                    &self.client_player_actions,
                    is_multiplayer,
                )),
                MobAction::Attack(MobAttackAction {
                    target,
                    attack_type,
                }) => match attack_type {
                    MobAttackType::Melee => Some(target_position(
                        *target,
                        &world_positions,
                        &net_world_positions,
                        &self.client_player_actions,
                        is_multiplayer,
                    )),
                    _ => Some(monster_position.position),
                },
                _ => None,
            }
        } else {
            match monster.action.action {
                MobAction::Chase(target) => Some(target_position(
                    target,
                    &world_positions,
                    &net_world_positions,
                    &self.client_player_actions,
                    is_multiplayer,
                )),
                _ => None,
            }
        };

        if let Some(destination) = new_destination {
            monster.destination = destination;
        }

        if let Some(action) = new_action {
            monster.action = Action {
                frame_number,
                action,
            }
        }
    }

    pub fn process_monster_movement(&self, entity: Entity, monster: &mut Monster) {
        let mut world_positions = self.world_positions.borrow_mut();
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

    fn new_action(
        &self,
        frame_number: u64,
        monster: &Monster,
        monster_position: WorldPosition,
    ) -> Option<MobAction<Entity>> {
        let players = self.players.borrow();
        let world_positions = self.world_positions.borrow();
        let mut damage_histories = self.damage_histories.borrow_mut();

        let monster_definition = self
            .monster_definitions
            .0
            .get(&monster.name)
            .expect("Expected a MonsterDefinition");

        match monster.action.action {
            MobAction::Idle => {
                if let Some((entity, _player_position)) = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
                    200.0,
                ) {
                    Some(MobAction::Chase(entity))
                } else {
                    let time_being_idle = self
                        .game_time_service
                        .seconds_between_frames(frame_number, monster.action.frame_number);
                    if MAX_IDLE_TIME_SECS < time_being_idle {
                        Some(MobAction::Move(random_scene_position(
                            &*self.game_level_state,
                        )))
                    } else {
                        None
                    }
                }
            }
            MobAction::Move(destination) => {
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
            MobAction::Chase(_) => {
                if let Some((target, _player_position)) = find_player_in_radius(
                    (self.entities, &*players, &*world_positions).join(),
                    *monster_position,
                    monster.radius,
                ) {
                    if self.game_state_helper.is_authoritative() {
                        let damage_history = damage_histories
                            .get_mut(target)
                            .expect("Expected player's DamageHistory");
                        damage_history.add_entry(
                            frame_number,
                            DamageHistoryEntry {
                                damage: monster.attack_damage,
                            },
                        );
                    }
                    Some(MobAction::Attack(MobAttackAction {
                        target,
                        attack_type: monster_definition.attack_type.randomize_params(0.2),
                    }))
                } else {
                    None
                }
            }
            MobAction::Attack(ref attack_action) => {
                let is_cooling_down = match attack_action.attack_type {
                    MobAttackType::SlowMelee { cooldown } => {
                        self.game_time_service
                            .seconds_between_frames(frame_number, monster.action.frame_number)
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

#[cfg(feature = "client")]
fn target_position(
    entity: Entity,
    world_positions: &WriteStorage<WorldPosition>,
    net_positions: &WriteStorage<NetWorldPosition>,
    client_player_actions: &ReadStorage<ClientPlayerActions>,
    is_multiplayer: bool,
) -> Vector2 {
    let is_controllable = client_player_actions.contains(entity);
    if is_multiplayer && is_controllable {
        **net_positions
            .get(entity)
            .expect("Expected a NetWorldPosition of a mob target")
    } else {
        **world_positions
            .get(entity)
            .expect("Expected a WorldPosition of a mob target")
    }
}

#[cfg(not(feature = "client"))]
fn target_position(
    entity: Entity,
    world_positions: &WriteStorage<WorldPosition>,
    _net_positions: &WriteStorage<NetWorldPosition>,
    _client_player_actions: &ReadStorage<ClientPlayerActions>,
    _is_multiplayer: bool,
) -> Vector2 {
    **world_positions
        .get(entity)
        .expect("Expected a WorldPosition of a mob target")
}

#[cfg(feature = "client")]
fn add_mob_action_update(
    _outcoming_net_updates: &mut OutcomingNetUpdates,
    _action: NetUpdateWithPosition<MobAction<NetIdentifier>>,
) {
}

#[cfg(not(feature = "client"))]
fn add_mob_action_update(
    outcoming_net_updates: &mut OutcomingNetUpdates,
    action: NetUpdateWithPosition<MobAction<NetIdentifier>>,
) {
    outcoming_net_updates.mob_actions_updates.push(action);
}
