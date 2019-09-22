use amethyst::{
    core::math::clamp,
    ecs::{Entity, ReadExpect, ReadStorage},
};

#[cfg(feature = "client")]
use ha_core::net::NetUpdate;
#[cfg(not(feature = "client"))]
use ha_core::net::NetUpdateWithPosition;
use ha_core::{
    actions::player::PlayerWalkAction,
    ecs::{
        components::{ClientPlayerActions, Player, PlayerActions, WorldPosition},
        resources::{
            net::{EntityNetMetadataStorage, MultiplayerGameState},
            GameLevelState,
        },
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
    net::EntityNetIdentifier,
};

use super::super::{OutcomingNetUpdates, WriteStorageCell};

pub struct PlayerActionSubsystem<'s> {
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub multiplayer_game_state: &'s ReadExpect<'s, MultiplayerGameState>,
    pub entity_net_metadata_service: &'s ReadExpect<'s, EntityNetMetadataStorage>,
    pub client_player_actions: &'s ReadStorage<'s, ClientPlayerActions>,
    pub player_actions: WriteStorageCell<'s, PlayerActions>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
}

pub struct ApplyWalkActionNetArgs<'a> {
    pub entity_net_id: EntityNetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    pub updates: Option<(Option<WorldPosition>, Option<PlayerWalkAction>)>,
}

const PLAYER_SPEED: f32 = 200.0;

impl<'s> PlayerActionSubsystem<'s> {
    pub fn apply_walk_action<'a>(
        &self,
        frame_number: u64,
        entity: Entity,
        player: &mut Player,
        net_args: Option<ApplyWalkActionNetArgs<'a>>,
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
        let client_walk_action = client_player_actions
            .as_ref()
            .map(|actions| actions.walk_action.clone());
        let is_controllable = client_walk_action.is_some();
        let is_latest_frame = self.game_time_service.game_frame_number() == frame_number;

        if self.multiplayer_game_state.is_playing {
            let ApplyWalkActionNetArgs {
                entity_net_id,
                outcoming_net_updates,
                updates,
            } = net_args.expect("Expected ApplyWalkActionNetArgs in multiplayer");

            let walk_action_update = if let Some((updated_position, updated_walk_action)) = updates
            {
                // Update position if it's an authoritative server update.
                if let Some(updated_position) = updated_position {
                    *player_position = updated_position.clone();
                }

                Some(self.actual_action(frame_number, updated_walk_action, client_walk_action))
            } else if let Some(client_walk_action) = client_walk_action {
                if self.game_time_service.game_frame_number() == frame_number {
                    Some(client_walk_action)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(walk_action_update) = walk_action_update {
                // Check whether walking direction changed indeed.
                let updated_direction = walk_action_update.as_ref().map(|action| action.direction);
                let is_new_direction = match &player_actions.walk_action.action {
                    Some(action) => updated_direction.map_or(true, |direction| {
                        (action.direction - direction).norm_squared() > 0.001
                    }),
                    None => updated_direction.is_some(),
                };

                if is_new_direction {
                    log::debug!(
                        "Applying a new walk update for {} (frame {}): {:?}",
                        entity_net_id,
                        frame_number,
                        &walk_action_update
                    );
                    // Update player actions.
                    player_actions.walk_action.frame_number = frame_number;
                    player_actions.walk_action.action = walk_action_update;

                    // Add to network broadcasted updates.
                    self.add_walk_action_net_update(
                        outcoming_net_updates,
                        entity_net_id,
                        player_position.clone(),
                        player_actions.walk_action.action.clone(),
                        is_controllable,
                        is_latest_frame,
                    );
                }
            }
        } else {
            player_actions.walk_action.frame_number = frame_number;
            player_actions.walk_action.action =
                client_walk_action.expect("Expected ClientPlayerActions in single player");
        }

        // Run player actions.
        if let Some(walk_action) = &player_actions.walk_action.action {
            player.walking_direction = walk_action.direction;
            player.velocity = if walk_action.direction != Vector2::zero() {
                walk_action.direction.normalize() * PLAYER_SPEED
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

    #[allow(clippy::option_option)] // sry
    pub fn actual_action<T>(
        &self,
        frame_number: u64,
        updated_player_action: Option<T>,
        client_player_action: Option<Option<T>>,
    ) -> Option<T> {
        if self.game_time_service.game_frame_number() == frame_number {
            if let Some(action) = client_player_action {
                return action;
            }
        }
        updated_player_action
    }

    #[cfg(feature = "client")]
    pub fn add_walk_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: EntityNetIdentifier,
        _player_position: WorldPosition,
        walk_action: Option<PlayerWalkAction>,
        is_controllable: bool,
        is_latest_frame: bool,
    ) {
        if is_controllable && is_latest_frame {
            outcoming_net_updates.walk_action_updates.push(NetUpdate {
                entity_net_id,
                data: walk_action,
            });
        }
    }

    #[cfg(not(feature = "client"))]
    pub fn add_walk_action_net_update(
        &self,
        outcoming_net_updates: &mut OutcomingNetUpdates,
        entity_net_id: EntityNetIdentifier,
        player_position: WorldPosition,
        walk_action: Option<PlayerWalkAction>,
        _is_controllable: bool,
        _is_latest_frame: bool,
    ) {
        outcoming_net_updates
            .player_walk_actions_updates
            .push(NetUpdateWithPosition {
                entity_net_id,
                position: player_position,
                data: walk_action,
            });
    }
}
