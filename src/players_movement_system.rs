use amethyst::{
    core::{Time, Transform},
    ecs::{Join, Read, System, WriteStorage},
    input::InputHandler,
};

use crate::components::{Player, WorldPosition};
use crate::{Vector2, Vector3};

pub struct PlayersMovementSystem;

const PLAYER_SPEED: f32 = 500.0;

impl<'s> System<'s> for PlayersMovementSystem {
    type SystemData = (
        Read<'s, Time>,
        Read<'s, InputHandler<String, String>>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (time, input, players, mut transforms, mut world_positions): Self::SystemData,
    ) {
        match (input.axis_value("horizontal"), input.axis_value("vertical")) {
            (Some(x), Some(y)) if x != 0.0 || y != 0.0 => {
                let (_player, transform, world_position) =
                    (&players, &mut transforms, &mut world_positions)
                        .join()
                        .next()
                        .unwrap();

                world_position.position += Vector2::new(x as f32, y as f32).normalize()
                    * PLAYER_SPEED
                    * time.delta_real_seconds();

                transform.set_translation(Vector3::new(
                    world_position.position.x,
                    world_position.position.y,
                    0.0,
                ));
            }
            _ => {}
        }
    }
}
