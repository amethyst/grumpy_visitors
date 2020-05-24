use amethyst::{
    core::{math::Point3, Parent, Transform},
    ecs::{
        Entities, Entity, Join, ReadExpect, ReadStorage, System, World, WriteExpect, WriteStorage,
    },
    input::{InputHandler, StringBindings},
    renderer::Camera,
    shred::{ResourceId, SystemData},
    window::ScreenDimensions,
    winit::MouseButton,
};

use gv_core::{
    actions::player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
    ecs::components::{ClientPlayerActions, WorldPosition},
    math::Vector2,
};
use gv_game::ecs::system_data::GameStateHelper;

use std::collections::HashSet;

use crate::ecs::resources::DisplayDebugInfoSettings;

#[derive(SystemData)]
pub struct InputSystemData<'s> {
    input: ReadExpect<'s, InputHandler<StringBindings>>,
    screen_dimensions: ReadExpect<'s, ScreenDimensions>,
    transforms: ReadStorage<'s, Transform>,
    display_debug_info_settings: WriteExpect<'s, DisplayDebugInfoSettings>,
}

#[derive(Default)]
pub struct InputSystem {
    down_actions: HashSet<String>,
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        GameStateHelper<'s>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, ClientPlayerActions>,
        InputSystemData<'s>,
    );

    fn run(
        &mut self,
        (
            game_state_helper,
            entities,
            cameras,
            parents,
            world_positions,
            mut client_player_actions,
            mut input_system_data,
        ): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        let (camera_entity, camera_parent, _) = (&entities, &parents, &cameras)
            .join()
            .next()
            .expect("Expected a Camera attached to a Player");
        let player_entity = camera_parent.entity;
        let client_player_actions = client_player_actions
            .get_mut(player_entity)
            .expect("Expected a ClientPlayerActions component");
        let player_position = world_positions
            .get(player_entity)
            .expect("Expected a WorldPosition");
        self.process_mouse_input(
            &mut input_system_data,
            camera_entity,
            &cameras,
            &mut *client_player_actions,
            **player_position,
        );
        self.process_keyboard_input(&mut input_system_data, &mut *client_player_actions);
    }
}

impl InputSystem {
    fn process_mouse_input(
        &mut self,
        system_data: &mut InputSystemData,
        camera_entity: Entity,
        cameras: &ReadStorage<'_, Camera>,
        client_player_actions: &mut ClientPlayerActions,
        player_position: Vector2,
    ) {
        let mouse_world_position = {
            let mouse_position = system_data.input.mouse_position();
            if mouse_position.is_none() {
                return;
            }
            let (mouse_x, mouse_y) = mouse_position.unwrap();

            let camera = cameras.get(camera_entity).expect("Expected a Camera");
            let camera_transform = system_data
                .transforms
                .get(camera_entity)
                .expect("Expected a Transform");

            let position = camera.projection().screen_to_world_point(
                Point3::new(mouse_x as f32, mouse_y as f32, 0.0),
                system_data.screen_dimensions.diagonal(),
                camera_transform,
            );
            Vector2::new(position.x, position.y)
        };

        client_player_actions.look_action = PlayerLookAction {
            direction: mouse_world_position - player_position,
        };

        if system_data.input.mouse_button_is_down(MouseButton::Left) {
            client_player_actions.cast_action = Some(PlayerCastAction {
                cast_position: player_position,
                target_position: mouse_world_position,
            });
        } else {
            client_player_actions.cast_action = None;
        }
    }

    fn process_keyboard_input(
        &mut self,
        system_data: &mut InputSystemData,
        client_player_actions: &mut ClientPlayerActions,
    ) {
        let direction = if let (Some(x), Some(y)) = (
            system_data.input.axis_value("horizontal"),
            system_data.input.axis_value("vertical"),
        ) {
            if x == 0.0 && y == 0.0 {
                None
            } else {
                Some(Vector2::new(x, y))
            }
        } else {
            None
        };

        let display_health = &mut system_data.display_debug_info_settings.display_health;
        self.process_toggle_action(&system_data.input, "toggle_healthbars", || {
            *display_health = !*display_health;
        });

        let display_network_debug_info = &mut system_data
            .display_debug_info_settings
            .display_network_debug_info;
        self.process_toggle_action(&system_data.input, "toggle_network_debug_info", || {
            *display_network_debug_info = !*display_network_debug_info;
        });

        #[cfg(feature = "profiler")]
        self.process_toggle_action(&system_data.input, "toggle_profiler", || {
            log::info!("Toggling profiler");
            thread_profiler::toggle_profiler();
        });

        let action = direction
            .map(|direction| PlayerWalkAction::Walk { direction })
            .unwrap_or(PlayerWalkAction::Stop);
        client_player_actions.walk_action = action;
    }

    fn process_toggle_action(
        &mut self,
        input: &InputHandler<StringBindings>,
        action: &str,
        handler: impl FnOnce(),
    ) {
        if input.action_is_down(action).unwrap_or_default() {
            if !self.down_actions.contains(action) {
                self.down_actions.insert(action.to_owned());
                handler();
            }
        } else {
            self.down_actions.remove(action);
        }
    }
}
