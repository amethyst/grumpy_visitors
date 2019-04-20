use amethyst::{
    core::{math::Point2, GlobalTransform, Transform},
    ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage},
    input::InputHandler,
    renderer::{Camera, Material, MeshHandle, MouseButton, ScreenDimensions},
};

use std::time::{Duration, Instant};

use crate::{
    components::{Missile, Player, WorldPosition},
    data_resources::MissileGraphics,
    factories::create_missile,
    Vector2,
};

pub struct InputSystem {
    last_spawned: Instant,
}

const SPAWN_COOLDOWN: Duration = Duration::from_millis(500);

impl InputSystem {
    pub fn new() -> Self {
        Self {
            last_spawned: Instant::now() - SPAWN_COOLDOWN,
        }
    }
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        ReadExpect<'s, InputHandler<String, String>>,
        ReadExpect<'s, ScreenDimensions>,
        Entities<'s>,
        ReadExpect<'s, MissileGraphics>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, MeshHandle>,
        WriteStorage<'s, Material>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, Missile>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, GlobalTransform>,
    );

    fn run(
        &mut self,
        (
            input,
            screen_dimensions,
            entities,
            missile_graphics,
            mut transforms,
            mut meshes,
            mut materials,
            mut world_positions,
            mut missiles,
            players,
            cameras,
            global_transforms,
        ): Self::SystemData,
    ) {
        let mouse_position = input.mouse_position();
        if let Some((mouse_x, mouse_y)) = mouse_position {
            let (camera, camera_global_transform) =
                (&cameras, &global_transforms).join().next().unwrap();
            let position = camera.position_from_screen(
                Point2::new(mouse_x as f32, mouse_y as f32),
                camera_global_transform,
                &screen_dimensions,
            );

            if input.mouse_button_is_down(MouseButton::Left) {
                let now = Instant::now();
                if now.duration_since(self.last_spawned) > SPAWN_COOLDOWN {
                    let (_, player_position) = (&players, &world_positions).join().next().unwrap();

                    create_missile(
                        Vector2::new(position.x, position.y),
                        player_position.position,
                        now,
                        entities.build_entity(),
                        missile_graphics.0.clone(),
                        &mut transforms,
                        &mut meshes,
                        &mut materials,
                        &mut world_positions,
                        &mut missiles,
                    );

                    self.last_spawned = now;
                }
            }
        }
    }
}
