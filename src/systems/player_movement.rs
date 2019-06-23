use amethyst::{
    core::{math::clamp, Float, Time},
    ecs::{Join, Read, ReadExpect, System, WriteStorage},
};

use crate::{
    components::{Player, PlayerActions, WorldPosition},
    data_resources::GameScene,
    Vector2, ZeroVector,
};

pub struct PlayerMovementSystem;

const PLAYER_SPEED: Float = Float::from_f32(200.0);

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        ReadExpect<'s, GameScene>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, PlayerActions>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, game_scene, mut players, mut player_actions, mut world_positions): Self::SystemData,
    ) {
        for (player, player_position, player_actions) in
            (&mut players, &mut world_positions, &mut player_actions).join()
        {
            if player_actions.walk_actions.is_empty() {
                player.velocity = Vector2::zero();
            } else {
                for walk_action in player_actions.walk_actions.drain(..) {
                    player.walking_direction = walk_action.direction;
                    player.velocity = if walk_action.direction != Vector2::zero() {
                        walk_action.direction.normalize() * PLAYER_SPEED
                    } else {
                        Vector2::zero()
                    }
                }
                **player_position += player.velocity * Float::from(time.delta_seconds());

                let scene_half_size_x = game_scene.dimensions.x / 2.0.into();
                let scene_half_size_y = game_scene.dimensions.y / 2.0.into();
                player_position.x = clamp(player_position.x, -scene_half_size_x, scene_half_size_x);
                player_position.y = clamp(player_position.y, -scene_half_size_y, scene_half_size_y);
            }

            for look_action in player_actions.look_actions.drain(..) {
                player.looking_direction = look_action.direction;
            }
        }
    }
}
