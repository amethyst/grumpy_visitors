use amethyst::{
    core::math::clamp,
    ecs::{Entity, ReadExpect, ReadStorage},
};

#[cfg(feature = "client")]
use ha_core::net::NetUpdate;
#[cfg(not(feature = "client"))]
use ha_core::net::NetUpdateWithPosition;
use ha_core::{
    actions::{player::PlayerWalkAction, ClientActionUpdate},
    ecs::{
        components::{ClientPlayerActions, Player, PlayerActions, WorldPosition},
        resources::{
            net::{ActionUpdateIdProvider, EntityNetMetadataStorage, MultiplayerGameState},
            GameLevelState,
        },
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
    net::NetIdentifier,
};

use super::super::{ClientFrameUpdate, OutcomingNetUpdates, WriteExpectCell, WriteStorageCell};

pub struct PlayerActionSubsystem<'s> {
    pub game_time_service: &'s GameTimeService<'s>,
    pub game_level_state: &'s ReadExpect<'s, GameLevelState>,
    pub multiplayer_game_state: &'s ReadExpect<'s, MultiplayerGameState>,
    pub entity_net_metadata_storage: &'s ReadExpect<'s, EntityNetMetadataStorage>,
    pub client_player_actions: &'s ReadStorage<'s, ClientPlayerActions>,
    pub action_update_id_provider: WriteExpectCell<'s, ActionUpdateIdProvider>,
    pub player_actions: WriteStorageCell<'s, PlayerActions>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
}

pub struct ApplyWalkActionNetArgs<'a> {
    pub entity_net_id: NetIdentifier,
    pub outcoming_net_updates: &'a mut OutcomingNetUpdates,
    pub updates: Option<(Option<WorldPosition>, ClientActionUpdate<PlayerWalkAction>)>,
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
                log::debug!(
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

    //    #[cfg(feature = "client")]
    //    pub fn apply_client_walk_action(&self, frame_number: u64, entity: Entity)

    #[cfg(feature = "client")]
    pub fn actual_walk_action_update(
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
                let client_action_id = action_update_id_provider.next_update_id();
                if *current_walk_action != new_client_walk_action {
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
    pub fn actual_walk_action_update(
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
    pub fn add_walk_action_net_update(
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
    pub fn add_walk_action_net_update(
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
}
