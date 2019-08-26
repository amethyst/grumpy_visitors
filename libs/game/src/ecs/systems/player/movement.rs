use amethyst::{
    core::math::clamp,
    ecs::{Join, ReadExpect, ReadStorage, System, WriteStorage},
};

use ha_core::{
    actions::player::{PlayerLookAction, PlayerWalkAction},
    ecs::{
        components::{Dead, Player, PlayerActions, WorldPosition},
        resources::{
            net::EntityNetMetadataService,
            world::{FramedUpdates, ServerWorldUpdate, WorldStates},
            GameLevelState,
        },
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
};

pub struct PlayerMovementSystem;

const PLAYER_SPEED: f32 = 200.0;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        GameTimeService<'s>,
        ReadExpect<'s, GameLevelState>,
        ReadExpect<'s, FramedUpdates<ServerWorldUpdate>>,
        ReadExpect<'s, WorldStates>,
        ReadExpect<'s, EntityNetMetadataService>,
        ReadStorage<'s, Dead>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, PlayerActions>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_scene,
            framed_updates,
            world_states,
            entity_net_metadata_service,
            dead,
            mut players,
            mut player_actions,
            mut world_positions,
        ): Self::SystemData,
    ) {
        let apply_walk_action =
            |player: &mut Player,
             player_position: &mut WorldPosition,
             walk_action: &Option<PlayerWalkAction>| {
                if let Some(walk_action) = walk_action {
                    player.walking_direction = walk_action.direction;
                    player.velocity = if walk_action.direction != Vector2::zero() {
                        walk_action.direction.normalize() * PLAYER_SPEED
                    } else {
                        Vector2::zero()
                    };
                    **player_position +=
                        player.velocity * game_time_service.engine_time().fixed_seconds();

                    let scene_half_size_x = game_scene.dimensions.x / 2.0;
                    let scene_half_size_y = game_scene.dimensions.y / 2.0;
                    player_position.x =
                        clamp(player_position.x, -scene_half_size_x, scene_half_size_x);
                    player_position.y =
                        clamp(player_position.y, -scene_half_size_y, scene_half_size_y);
                } else {
                    player.velocity = Vector2::zero();
                }
            };

        let apply_look_action = |player: &mut Player, look_action: &PlayerLookAction| {
            player.looking_direction = look_action.direction;
        };

        assert!(world_states.can_apply_updates(&framed_updates));
        // Run each updated frame.
        for frame_updated in framed_updates.iter_from_oldest_update() {
            // Update no further than a current frame.
            if game_time_service.game_frame_number() < frame_updated.frame_number {
                break;
            }

            for player_actions_updates in &frame_updated.player_actions_updates {
                // Restore the previous world state.

                // Apply server updates.
                let entity = entity_net_metadata_service
                    .get_entity(player_actions_updates.entity_net_identifier);
                let player = players.get_mut(entity).expect("Expected a Player");
                let player_position = world_positions
                    .get_mut(entity)
                    .expect("Expected a WorldPosition");
                *player_position = player_actions_updates.position.clone();

                apply_walk_action(
                    player,
                    player_position,
                    &player_actions_updates.data.walk_action.action,
                );

                if let Some(look_action) = player_actions_updates.data.look_action.action.as_ref() {
                    apply_look_action(player, look_action);
                }
            }
        }

        // Apply this frame client updates.
        for (player, player_position, player_actions, _) in (
            &mut players,
            &mut world_positions,
            &mut player_actions,
            !&dead,
        )
            .join()
        {
            apply_walk_action(player, player_position, &player_actions.walk_action.action);

            if let Some(look_action) = player_actions.look_action.action.as_ref() {
                apply_look_action(player, look_action);
            }
        }
    }
}
