use amethyst::{
    assets::Handle,
    core::{math::Point2, Time, Transform},
    ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::{Camera, Material, Mesh},
    window::ScreenDimensions,
};
use winit::MouseButton;

use std::time::Duration;

use crate::{
    components::{Missile, Player, WorldPosition},
    data_resources::MissileGraphics,
    factories::create_missile,
    models::GameState,
    utils::camera,
    Vector2,
};

pub struct InputSystem {
    last_spawned: Duration,
}

const SPAWN_COOLDOWN: Duration = Duration::from_millis(500);

impl InputSystem {
    pub fn new() -> Self {
        Self {
            last_spawned: Duration::new(0, 0),
        }
    }
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        ReadExpect<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, Time>,
        Entities<'s>,
        ReadExpect<'s, GameState>,
        ReadExpect<'s, MissileGraphics>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Mesh>>,
        WriteStorage<'s, Handle<Material>>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, Missile>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Camera>,
    );

    fn run(
        &mut self,
        (
            input,
            screen_dimensions,
            time,
            entities,
            game_state,
            missile_graphics,
            mut transforms,
            mut meshes,
            mut materials,
            mut world_positions,
            mut missiles,
            mut players,
            cameras,
        ): Self::SystemData,
    ) {
        let mouse_position = input.mouse_position();
        if mouse_position.is_none() {
            return;
        }
        let (mouse_x, mouse_y) = mouse_position.unwrap();

        let components = (&cameras, &transforms).join().next();
        if components.is_none() {
            return;
        }
        let (camera, camera_transform) = components.unwrap();

        let position = camera::screen_to_world(
            &camera,
            Point2::new(mouse_x as f32, mouse_y as f32),
            camera_transform,
            &screen_dimensions,
        );

        if let GameState::Playing = *game_state {
        } else {
            return;
        }

        let (mut player, player_position) = (&mut players, &world_positions).join().next().unwrap();
        player.looking_direction = Vector2::new(position.x, position.y) - **player_position;

        if input.mouse_button_is_down(MouseButton::Left) {
            let now = time.absolute_time();
            if now - self.last_spawned > SPAWN_COOLDOWN {
                create_missile(
                    Vector2::new(position.x, position.y),
                    **player_position,
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
