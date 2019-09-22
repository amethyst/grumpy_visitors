use amethyst::{
    core::{math::Point2, Parent, Transform},
    ecs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::Camera,
    window::ScreenDimensions,
    winit::MouseButton,
};

use ha_core::{
    actions::player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
    ecs::{
        components::{ClientPlayerActions, WorldPosition},
        system_data::game_state_helper::GameStateHelper,
    },
    math::Vector2,
};

#[derive(Default)]
pub struct InputSystem;

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        GameStateHelper<'s>,
        Entities<'s>,
        ReadExpect<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, ClientPlayerActions>,
    );

    fn run(
        &mut self,
        (
            game_state_helper,
            entities,
            input,
            screen_dimensions,
            cameras,
            parents,
            transforms,
            world_positions,
            mut client_player_actions,
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
            &screen_dimensions,
            &*input,
            camera_entity,
            &cameras,
            &transforms,
            &mut *client_player_actions,
            **player_position,
        );
        self.process_keyboard_input(&*input, &mut *client_player_actions);
    }
}

impl InputSystem {
    fn process_mouse_input(
        &mut self,
        screen_dimensions: &ScreenDimensions,
        input: &InputHandler<StringBindings>,
        camera_entity: Entity,
        cameras: &ReadStorage<'_, Camera>,
        transforms: &ReadStorage<'_, Transform>,
        client_player_actions: &mut ClientPlayerActions,
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

        client_player_actions.look_action = Some(PlayerLookAction {
            direction: mouse_world_position - player_position,
        });

        if input.mouse_button_is_down(MouseButton::Left) {
            client_player_actions.cast_action = Some(PlayerCastAction {
                cast_position: player_position,
                target_position: mouse_world_position,
            });
        }
    }

    fn process_keyboard_input(
        &mut self,
        input: &InputHandler<StringBindings>,
        client_player_actions: &mut ClientPlayerActions,
    ) {
        let direction = if let (Some(x), Some(y)) =
            (input.axis_value("horizontal"), input.axis_value("vertical"))
        {
            if x == 0.0 && y == 0.0 {
                None
            } else {
                Some(Vector2::new(x, y))
            }
        } else {
            None
        };

        let action = direction.map(|direction| PlayerWalkAction { direction });
        client_player_actions.walk_action = action;
    }
}
