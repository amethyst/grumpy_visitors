use amethyst::{
    core::{math::Point2, Parent, Transform},
    ecs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::Camera,
    window::ScreenDimensions,
    winit::MouseButton,
};

use ha_core::{
    actions::{
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
        Action,
    },
    ecs::{
        components::{PlayerActions, WorldPosition},
        resources::GameEngineState,
        system_data::time::GameTimeService,
    },
    math::Vector2,
};

#[derive(Default)]
pub struct InputSystem;

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        Entities<'s>,
        GameTimeService<'s>,
        ReadExpect<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameEngineState>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, PlayerActions>,
    );

    fn run(
        &mut self,
        (
            entities,
            game_time_service,
            input,
            screen_dimensions,
            game_state,
            cameras,
            parents,
            transforms,
            world_positions,
            mut player_actions,
        ): Self::SystemData,
    ) {
        if let GameEngineState::Playing = *game_state {
        } else {
            return;
        }

        let (camera_entity, camera_parent, _) = (&entities, &parents, &cameras)
            .join()
            .next()
            .expect("Expected a Camera attached to a Player");
        let player_entity = camera_parent.entity;
        let player_actions = player_actions
            .get_mut(player_entity)
            .expect("Expected PlayerActions");
        let player_position = world_positions
            .get(player_entity)
            .expect("Expected a WorldPosition");
        self.process_mouse_input(
            &game_time_service,
            &screen_dimensions,
            &*input,
            camera_entity,
            &cameras,
            &transforms,
            &mut *player_actions,
            **player_position,
        );
        self.process_keyboard_input(&game_time_service, &*input, &mut *player_actions);
    }
}

impl InputSystem {
    fn process_mouse_input(
        &mut self,
        game_time_service: &GameTimeService,
        screen_dimensions: &ScreenDimensions,
        input: &InputHandler<StringBindings>,
        camera_entity: Entity,
        cameras: &ReadStorage<'_, Camera>,
        transforms: &ReadStorage<'_, Transform>,
        player_actions: &mut PlayerActions,
        player_position: Vector2,
    ) {
        let mouse_world_position = {
            let mouse_position = input.mouse_position();
            if mouse_position.is_none() {
                return;
            }
            let (mouse_x, mouse_y) = mouse_position.unwrap();

            let camera = cameras.get(camera_entity).expect("Expected a Camera");
            let camera_transform = transforms.get(camera_entity).expect("Expected a Transform");

            let position = camera.projection().screen_to_world(
                Point2::new(mouse_x as f32, mouse_y as f32),
                screen_dimensions.diagonal(),
                camera_transform,
            );
            Vector2::new(position.x, position.y)
        };

        player_actions.look_action = Action {
            frame_number: game_time_service.game_frame_number(),
            action: Some(PlayerLookAction {
                direction: mouse_world_position - player_position,
            }),
        };

        if input.mouse_button_is_down(MouseButton::Left) {
            player_actions.cast_action = Action {
                frame_number: game_time_service.game_frame_number(),
                action: Some(PlayerCastAction {
                    cast_position: player_position,
                    target_position: mouse_world_position,
                }),
            };
        }
    }

    fn process_keyboard_input(
        &mut self,
        game_time_service: &GameTimeService,
        input: &InputHandler<StringBindings>,
        player_actions: &mut PlayerActions,
    ) {
        if let (Some(x), Some(y)) = (input.axis_value("horizontal"), input.axis_value("vertical")) {
            player_actions.walk_action = Action {
                frame_number: game_time_service.game_frame_number(),
                action: Some(PlayerWalkAction {
                    direction: Vector2::new(x, y),
                }),
            };
        }
    }
}
