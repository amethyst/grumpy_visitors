use amethyst::{
    core::math::clamp,
    ecs::{Entities, Entity, Join, ReadExpect, ReadStorage},
};

use std::time::Duration;

use crate::ecs::system_data::GameStateHelper;
#[cfg(not(feature = "client"))]
use ha_core::net::NetUpdateWithPosition;
use ha_core::{
    actions::{
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
        ClientActionUpdate, IdentifiableAction,
    },
    ecs::{
        components::{
            missile::Missile, ClientPlayerActions, Player, PlayerActions, PlayerLastCastedSpells,
            WorldPosition,
        },
        resources::{
            net::{ActionUpdateIdProvider, CastActionsToExecute, MultiplayerGameState},
            GameLevelState,
        },
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
    net::{NetIdentifier, NetUpdate},
};

use super::super::{ClientFrameUpdate, OutcomingNetUpdates, WriteExpectCell, WriteStorageCell};

const MISSILE_CAST_COOLDOWN: Duration = Duration::from_millis(500);

pub struct PlayerActionSubsystem<'s> {
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_state_helper: &'s GameStateHelper<'s>,
    pub entities: &'s Entities<'s>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub multiplayer_game_state: &'s ReadExpect<'s, MultiplayerGameState>,
    pub client_player_actions: &'s ReadStorage<'s, ClientPlayerActions>,
    pub action_update_id_provider: WriteExpectCell<'s, ActionUpdateIdProvider>,
    pub cast_actions_to_execute: WriteExpectCell<'s, CastActionsToExecute>,
    pub player_actions: WriteStorageCell<'s, PlayerActions>,
    pub player_last_casted_spells: WriteStorageCell<'s, PlayerLastCastedSpells>,
    pub missiles: WriteStorageCell<'s, Missile>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
}

pub struct ApplyWalkActionNetArgs<'a> {
    pub entity_net_id: NetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    /// If there's an update, this field will have WorldPosition on client side
    /// (as it receives such updates from server), and both client and server will have
    /// PlayerWalkAction update.
    pub updates: Option<(Option<WorldPosition>, ClientActionUpdate<PlayerWalkAction>)>,
}

pub struct ApplyLookActionNetArgs<'a> {
    pub entity_net_id: NetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    pub update: Option<ClientActionUpdate<PlayerLookAction>>,
}

pub struct ApplyCastActionNetArgs<'a> {
    pub entity_net_id: NetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    pub update: Option<IdentifiableAction<ClientActionUpdate<PlayerCastAction>>>,
}

const PLAYER_SPEED: f32 = 200.0;

impl<'s> PlayerActionSubsystem<'s> {
    pub fn apply_walk_action<'a>(
        &self,
        frame_number: u64,
        entity: Entity,
        player: &mut Player,
        net_args: Option<ApplyWalkActionNetArgs<'a>>,
        client_side_actions: &mut ClientFrameUpdate,
    ) {
        let mut world_positions = self.world_positions.borrow_mut();
        let player_position = world_positions
            .get_mut(entity)
            .expect("Expected a WorldPosition");

        let mut player_actions = self.player_actions.borrow_mut();
        let player_actions = player_actions
            .get_mut(entity)
            .expect("Expected player actions");

        let client_player_actions = self.client_player_actions.get(entity);
        let new_client_walk_action = client_player_actions
            .as_ref()
            .map(|actions| actions.walk_action.clone());
        let is_controllable = new_client_walk_action.is_some();
        let is_latest_frame = self.game_time_service.game_frame_number() == frame_number;

        if self.multiplayer_game_state.is_playing {
            let ApplyWalkActionNetArgs {
                entity_net_id,
                outcoming_net_updates,
                updates,
            } = net_args.expect("Expected ApplyWalkActionNetArgs in multiplayer");

            // Update position if it's an authoritative server update.
            if let Some(updated_position) = updates
                .clone()
                .and_then(|(updated_position, _)| updated_position)
            {
                *player_position = updated_position;
            }

            // Decide which source has an actual update and retrieve it.
            let updated_walk_action = updates.map(|(_, updated_action)| updated_action);
            let walk_action_update = self.actual_walk_action_update(
                frame_number,
                updated_walk_action,
                &player_actions.walk_action,
                new_client_walk_action,
                client_side_actions,
                entity_net_id,
            );

            if let Some(walk_action_update) = walk_action_update {
                log::trace!(
                    "Applying a new walk update for {} (frame {}): {:?}",
                    entity_net_id,
                    frame_number,
                    &walk_action_update
                );
                // Update player actions.
                player_actions.walk_action = walk_action_update.action.clone();

                // Add to network broadcasted updates.
                self.add_walk_action_net_update(
                    outcoming_net_updates,
                    entity_net_id,
                    player_position.clone(),
                    walk_action_update,
                    is_controllable,
                    is_latest_frame,
                );
            }
        } else {
            player_actions.walk_action =
                new_client_walk_action.expect("Expected ClientPlayerActions in single player");
        }

        // Run player actions.
        if let PlayerWalkAction::Walk { direction } = &player_actions.walk_action {
            player.walking_direction = *direction;
            player.velocity = if *direction != Vector2::zero() {
                direction.normalize() * PLAYER_SPEED
            } else {
                Vector2::zero()
            };
            **player_position +=
                player.velocity * self.game_time_service.engine_time().fixed_seconds();

            let scene_half_size_x = self.game_level_state.dimensions.x / 2.0;
            let scene_half_size_y = self.game_level_state.dimensions.y / 2.0;
            player_position.x = clamp(player_position.x, -scene_half_size_x, scene_half_size_x);
            player_position.y = clamp(player_position.y, -scene_half_size_y, scene_half_size_y);
        } else {
            player.velocity = Vector2::zero();
        }
    }

    pub fn apply_look_action<'a>(
        &self,
        frame_number: u64,
        entity: Entity,
        player: &mut Player,
        net_args: Option<ApplyLookActionNetArgs<'a>>,
        client_side_actions: &mut ClientFrameUpdate,
    ) {
        let mut player_actions = self.player_actions.borrow_mut();
        let player_actions = player_actions
            .get_mut(entity)
            .expect("Expected player actions");

        let client_player_actions = self.client_player_actions.get(entity);
        let new_client_look_action = client_player_actions
            .as_ref()
            .map(|actions| actions.look_action.clone());
        let is_controllable = new_client_look_action.is_some();
        let is_latest_frame = self.game_time_service.game_frame_number() == frame_number;

        if self.multiplayer_game_state.is_playing {
            let ApplyLookActionNetArgs {
                entity_net_id,
                outcoming_net_updates,
                update: updated_look_action,
            } = net_args.expect("Expected ApplyLookActionNetArgs in multiplayer");
            // Decide which source has an actual update and retrieve it.
            let look_action_update = self.actual_look_action_update(
                frame_number,
                updated_look_action,
                &player_actions.look_action,
                new_client_look_action,
                client_side_actions,
                entity_net_id,
            );

            if let Some(look_action_update) = look_action_update {
                log::trace!(
                    "Applying a new look update for {} (frame {}): {:?}",
                    entity_net_id,
                    frame_number,
                    &look_action_update
                );
                // Update player actions.
                player_actions.look_action = look_action_update.action.clone();

                // Add to network broadcasted updates.
                self.add_look_action_net_update(
                    outcoming_net_updates,
                    entity_net_id,
                    look_action_update,
                    frame_number,
                    is_controllable,
                    is_latest_frame,
                );
            }
        } else {
            player_actions.look_action =
                new_client_look_action.expect("Expected ClientPlayerActions in single player");
        }

        // Run player actions.
        player.looking_direction = player_actions.look_action.direction;
    }

    pub fn apply_cast_action<'a>(
        &self,
        frame_number: u64,
        entity: Entity,
        mut net_args: Option<ApplyCastActionNetArgs<'a>>,
        _client_side_actions: &mut ClientFrameUpdate,
    ) {
        let mut player_actions = self.player_actions.borrow_mut();
        let player_actions = player_actions
            .get_mut(entity)
            .expect("Expected player actions");

        let mut player_last_casted_spells = self.player_last_casted_spells.borrow_mut();
        let player_last_casted_spells = player_last_casted_spells
            .get_mut(entity)
            .expect("Expected PlayerLastCastedSpells component");

        let mut world_positions = self.world_positions.borrow_mut();
        let player_position = world_positions
            .get_mut(entity)
            .expect("Expected a WorldPosition")
            .clone();

        let mut cast_actions_to_execute = self.cast_actions_to_execute.borrow_mut();

        let client_player_actions = self.client_player_actions.get(entity);

        let is_latest_frame = self.game_time_service.game_frame_number() == frame_number;
        let is_cooling_down = self
            .game_time_service
            .seconds_between_frames(frame_number, player_last_casted_spells.missile)
            < MISSILE_CAST_COOLDOWN.as_secs_f32();

        player_actions.cast_action = None;

        if self.multiplayer_game_state.is_playing {
            let ApplyCastActionNetArgs {
                entity_net_id,
                outcoming_net_updates,
                update: cast_action_update,
            } = net_args
                .as_mut()
                .expect("Expected ApplyCastActionNetArgs in multiplayer");

            if let Some(IdentifiableAction {
                action_id,
                action: mut cast_action,
            }) = cast_action_update.clone()
            {
                if !is_cooling_down || !self.game_state_helper.is_authoritative() {
                    log::trace!(
                        "Applying a new cast update ({}) for {} (frame {}): {:?}",
                        action_id,
                        entity_net_id,
                        frame_number,
                        &cast_action
                    );
                }

                if self.game_state_helper.is_authoritative() && !is_cooling_down {
                    // Update player actions.
                    player_last_casted_spells.missile = frame_number;
                    cast_action.action.cast_position = *player_position;
                    player_actions.cast_action = Some(cast_action.action.clone());

                    // Add to network broadcasted updates.
                    self.add_cast_action_net_update(
                        outcoming_net_updates,
                        *entity_net_id,
                        Some(action_id),
                        cast_action,
                    );
                } else if !self.game_state_helper.is_authoritative() {
                    player_last_casted_spells.missile = frame_number;
                    player_actions.cast_action = Some(cast_action.action);
                }

                if let Some(cast_action) = &player_actions.cast_action {
                    if let Some(missile) = self.already_casted_missile(action_id) {
                        let missile_position = world_positions
                            .get_mut(missile)
                            .expect("Expected a WorldPosition for a Missile");
                        **missile_position = cast_action.cast_position;
                    } else {
                        cast_actions_to_execute.actions.push(IdentifiableAction {
                            action_id,
                            action: cast_action.clone(),
                        });
                    }

                    return;
                }
            }
        }

        if let Some(client_player_actions) = client_player_actions.cloned() {
            if is_latest_frame {
                if let Some(mut cast_action) = client_player_actions.cast_action {
                    if !is_cooling_down {
                        if self.multiplayer_game_state.is_playing {
                            let ApplyCastActionNetArgs {
                                entity_net_id,
                                outcoming_net_updates,
                                ..
                            } = net_args.expect("Expected ApplyCastActionNetArgs in multiplayer");
                            log::trace!(
                                "Sending a new cast update (for {}) to a server (frame {}): {:?}",
                                entity_net_id,
                                frame_number,
                                &cast_action
                            );

                            let mut action_update_id_provider =
                                self.action_update_id_provider.borrow_mut();
                            cast_action.cast_position = *player_position;

                            self.add_cast_action_net_update(
                                outcoming_net_updates,
                                entity_net_id,
                                None,
                                ClientActionUpdate {
                                    client_action_id: action_update_id_provider.next_update_id(),
                                    action: cast_action.clone(),
                                },
                            );
                        } else {
                            log::trace!(
                                "Applying a new cast update for {} (frame {}): {:?}",
                                entity.id(),
                                frame_number,
                                &cast_action
                            );
                            cast_actions_to_execute.actions.push(IdentifiableAction {
                                action_id: 0,
                                action: cast_action.clone(),
                            });
                        }

                        player_actions.cast_action = Some(cast_action);
                    }
                }
            }
            if player_actions.cast_action.is_some() {
                player_last_casted_spells.missile = frame_number;
            }
        }
    }

    #[cfg(feature = "client")]
    fn actual_walk_action_update(
        &self,
        frame_number: u64,
        updated_player_action: Option<ClientActionUpdate<PlayerWalkAction>>,
        current_walk_action: &PlayerWalkAction,
        new_client_walk_action: Option<PlayerWalkAction>,
        client_side_actions: &mut ClientFrameUpdate,
        entity_net_id: NetIdentifier,
    ) -> Option<ClientActionUpdate<PlayerWalkAction>> {
        if let Some(new_client_walk_action) = new_client_walk_action {
            if self.game_time_service.game_frame_number() == frame_number {
                let mut action_update_id_provider = self.action_update_id_provider.borrow_mut();
                if *current_walk_action != new_client_walk_action {
                    let client_action_id = action_update_id_provider.next_update_id();
                    let client_action_update = ClientActionUpdate {
                        client_action_id,
                        action: new_client_walk_action,
                    };
                    client_side_actions.walk_action_updates.push(NetUpdate {
                        entity_net_id,
                        data: client_action_update.clone(),
                    });
                    return Some(client_action_update);
                }
            }
        }
        updated_player_action.or_else(|| {
            client_side_actions
                .walk_action_updates
                .iter()
                .find(|action_update| action_update.entity_net_id == entity_net_id)
                .map(|client_side_action| client_side_action.data.clone())
        })
    }

    #[cfg(not(feature = "client"))]
    fn actual_walk_action_update(
        &self,
        _frame_number: u64,
        updated_player_action: Option<ClientActionUpdate<PlayerWalkAction>>,
        _current_walk_action: &PlayerWalkAction,
        _new_client_walk_action: Option<PlayerWalkAction>,
        _client_side_actions: &mut ClientFrameUpdate,
        _entity_net_id: NetIdentifier,
    ) -> Option<ClientActionUpdate<PlayerWalkAction>> {
        updated_player_action
    }

    #[cfg(feature = "client")]
    fn add_walk_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        _player_position: WorldPosition,
        walk_action_update: ClientActionUpdate<PlayerWalkAction>,
        is_controllable: bool,
        is_latest_frame: bool,
    ) {
        if is_controllable && is_latest_frame {
            outcoming_net_updates.walk_action_updates.push(NetUpdate {
                entity_net_id,
                data: walk_action_update,
            });
        }
    }

    #[cfg(not(feature = "client"))]
    fn add_walk_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        player_position: WorldPosition,
        walk_action_update: ClientActionUpdate<PlayerWalkAction>,
        _is_controllable: bool,
        _is_latest_frame: bool,
    ) {
        outcoming_net_updates
            .player_walk_actions_updates
            .push(NetUpdateWithPosition {
                entity_net_id,
                position: player_position,
                data: walk_action_update,
            });
    }

    #[cfg(feature = "client")]
    fn actual_look_action_update(
        &self,
        frame_number: u64,
        updated_player_action: Option<ClientActionUpdate<PlayerLookAction>>,
        current_look_action: &PlayerLookAction,
        new_client_look_action: Option<PlayerLookAction>,
        client_side_actions: &mut ClientFrameUpdate,
        entity_net_id: NetIdentifier,
    ) -> Option<ClientActionUpdate<PlayerLookAction>> {
        if let Some(new_client_look_action) = new_client_look_action {
            if self.game_time_service.game_frame_number() == frame_number {
                let mut action_update_id_provider = self.action_update_id_provider.borrow_mut();
                if *current_look_action != new_client_look_action {
                    let client_action_id = action_update_id_provider.next_update_id();
                    let client_action_update = ClientActionUpdate {
                        client_action_id,
                        action: new_client_look_action,
                    };
                    client_side_actions.look_action_updates.push(NetUpdate {
                        entity_net_id,
                        data: client_action_update.clone(),
                    });
                    return Some(client_action_update);
                }
            }
        }
        updated_player_action.or_else(|| {
            client_side_actions
                .look_action_updates
                .iter()
                .find(|action_update| action_update.entity_net_id == entity_net_id)
                .map(|client_side_action| client_side_action.data.clone())
        })
    }

    #[cfg(not(feature = "client"))]
    fn actual_look_action_update(
        &self,
        _frame_number: u64,
        updated_player_action: Option<ClientActionUpdate<PlayerLookAction>>,
        _current_look_action: &PlayerLookAction,
        _new_client_look_action: Option<PlayerLookAction>,
        _client_side_actions: &mut ClientFrameUpdate,
        _entity_net_id: NetIdentifier,
    ) -> Option<ClientActionUpdate<PlayerLookAction>> {
        updated_player_action
    }

    #[cfg(feature = "client")]
    fn add_look_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        look_action_update: ClientActionUpdate<PlayerLookAction>,
        frame_number: u64,
        is_controllable: bool,
        is_latest_frame: bool,
    ) {
        if is_controllable && is_latest_frame {
            let has_last_frame = outcoming_net_updates
                .look_actions_updates
                .back()
                .map_or(false, |(update_frame_number, _)| {
                    *update_frame_number == frame_number
                });
            if !has_last_frame {
                outcoming_net_updates
                    .look_actions_updates
                    .push_back((frame_number, Vec::with_capacity(1)));
            }

            outcoming_net_updates
                .look_actions_updates
                .back_mut()
                .unwrap()
                .1
                .push(NetUpdate {
                    entity_net_id,
                    data: look_action_update,
                });
        }
    }

    #[cfg(not(feature = "client"))]
    fn add_look_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        look_action_update: ClientActionUpdate<PlayerLookAction>,
        _frame_number: u64,
        _is_controllable: bool,
        _is_latest_frame: bool,
    ) {
        outcoming_net_updates
            .player_look_actions_updates
            .push(NetUpdate {
                entity_net_id,
                data: look_action_update,
            });
    }

    #[cfg(feature = "client")]
    fn add_cast_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        _action_id: Option<NetIdentifier>,
        cast_action_update: ClientActionUpdate<PlayerCastAction>,
    ) {
        outcoming_net_updates.cast_action_updates.push(NetUpdate {
            entity_net_id,
            data: cast_action_update,
        });
    }

    #[cfg(not(feature = "client"))]
    fn add_cast_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: NetIdentifier,
        action_id: Option<NetIdentifier>,
        cast_action_update: ClientActionUpdate<PlayerCastAction>,
    ) {
        outcoming_net_updates
            .player_cast_actions_updates
            .push(NetUpdate {
                entity_net_id,
                data: IdentifiableAction {
                    action_id: action_id.expect("Expected an action id passed for server"),
                    action: cast_action_update,
                },
            });
    }

    fn already_casted_missile(&self, cast_action_id: NetIdentifier) -> Option<Entity> {
        let missiles = self.missiles.borrow();
        (&*missiles, self.entities)
            .join()
            .find(|(missile, _)| missile.action_id == cast_action_id)
            .map(|(_, entity)| entity)
    }
}
