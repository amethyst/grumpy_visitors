use amethyst::{
    core::{math::clamp, Time, Transform},
    ecs::{Join, Read, ReadExpect, System, WriteStorage},
    input::InputHandler,
};

use crate::{
    components::{Player, WorldPosition},
    data_resources::GameScene,
    Vector2, Vector3,
};

pub struct PlayersMovementSystem;

const PLAYER_SPEED: f32 = 500.0;

impl<'s> System<'s> for PlayersMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        ReadExpect<'s, InputHandler<String, String>>,
        ReadExpect<'s, GameScene>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, input, game_scene, players, mut transforms, mut world_positions): Self::SystemData,
    ) {
        match (input.axis_value("horizontal"), input.axis_value("vertical")) {
            (Some(x), Some(y)) if x != 0.0 || y != 0.0 => {
                let (_player, transform, world_position) =
                    (&players, &mut transforms, &mut world_positions)
                        .join()
                        .next()
                        .unwrap();

                let world_position = &mut world_position.position;
                *world_position += Vector2::new(x as f32, y as f32).normalize()
                    * PLAYER_SPEED
                    * time.delta_real_seconds();

                let scene_half_size_x = game_scene.dimensions.x / 2.0;
                let scene_half_size_y = game_scene.dimensions.y / 2.0;
                world_position.x = clamp(world_position.x, -scene_half_size_x, scene_half_size_x);
                world_position.y = clamp(world_position.y, -scene_half_size_y, scene_half_size_y);

                transform.set_translation(Vector3::new(world_position.x, world_position.y, 0.0));
            }
            _ => {}
        }
    }
}
