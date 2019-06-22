use amethyst::{
    core::{math::clamp, Float, Time, Transform},
    ecs::{Join, Read, ReadExpect, System, WriteStorage},
    input::{InputHandler, StringBindings},
};

use crate::{
    components::{Player, WorldPosition},
    data_resources::GameScene,
    Vector2, Vector3,
};

pub struct PlayerMovementSystem;

const PLAYER_SPEED: Float = Float::from_f32(500.0);

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        ReadExpect<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, GameScene>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, input, game_scene, mut players, mut transforms, mut world_positions): Self::SystemData,
    ) {
        let components = (&mut players, &mut transforms, &mut world_positions)
            .join()
            .next();
        if components.is_none() {
            return;
        }
        let (player, transform, world_position) = components.unwrap();

        match (input.axis_value("horizontal"), input.axis_value("vertical")) {
            (Some(x), Some(y)) if x != 0.0 || y != 0.0 => {
                player.velocity =
                    Vector2::new(Float::from(x), Float::from(y)).normalize() * PLAYER_SPEED;
                player.walking_direction = player.velocity;

                let world_position = &mut **world_position;
                *world_position += player.velocity * Float::from(time.delta_real_seconds());

                let scene_half_size_x = game_scene.dimensions.x / 2.0.into();
                let scene_half_size_y = game_scene.dimensions.y / 2.0.into();
                world_position.x = clamp(world_position.x, -scene_half_size_x, scene_half_size_x);
                world_position.y = clamp(world_position.y, -scene_half_size_y, scene_half_size_y);

                transform.set_translation(Vector3::new(
                    world_position.x,
                    world_position.y,
                    transform.translation().z,
                ));
            }
            _ => {
                player.velocity = Vector2::new(0.0.into(), 0.0.into());
            }
        }
    }
}
