use amethyst::{
    core::{
        math::Point2,
        transform::{GlobalTransform, Parent, Transform},
    },
    ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage},
    renderer::{Camera, ScreenDimensions},
};
use num::Float;

use crate::{data_resources::GameScene, Vector2, Vector3};

pub struct CameraTranslationSystem;

impl<'s> System<'s> for CameraTranslationSystem {
    type SystemData = (
        ReadExpect<'s, GameScene>,
        ReadExpect<'s, ScreenDimensions>,
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        (
            game_scene,
            screen_dimensions,
            entities,
            parents,
            cameras,
            mut transforms,
        ): Self::SystemData,
    ) {
        let components = (&cameras, &parents, &entities).join().next();
        if components.is_none() {
            return;
        }
        let (camera, camera_parent, camera_id) = components.unwrap();
        let mut relaxed_camera_transform = transforms.get(camera_parent.entity).unwrap().clone();
        relaxed_camera_transform.set_translation(
            relaxed_camera_transform.translation()
                - Vector3::new(
                    screen_dimensions.width() / 2.0,
                    screen_dimensions.height() / 2.0,
                    0.0,
                ) / screen_dimensions.hidpi_factor() as f32,
        );
        let relaxed_camera_transform = GlobalTransform(relaxed_camera_transform.matrix());

        let screen_left_bottom = camera.position_from_screen(
            Point2::new(0.0, screen_dimensions.height()),
            &relaxed_camera_transform,
            &screen_dimensions,
        );
        let screen_left_bottom = Vector2::new(screen_left_bottom.x, screen_left_bottom.y);
        let screen_right_top = camera.position_from_screen(
            Point2::new(screen_dimensions.width(), 0.0),
            &relaxed_camera_transform,
            &screen_dimensions,
        );
        let screen_right_top = Vector2::new(screen_right_top.x, screen_right_top.y);

        let left_bottom_distance = -screen_left_bottom - game_scene.half_size();
        let right_top_distance = screen_right_top - game_scene.half_size();

        let camera_translation = -Vector2::new(
            screen_dimensions.width() / 2.0,
            screen_dimensions.height() / 2.0,
        ) / screen_dimensions.hidpi_factor() as f32
            + Vector2::new(
                Float::max(0.0, left_bottom_distance.x),
                Float::max(0.0, left_bottom_distance.y),
            )
            - Vector2::new(
                Float::max(0.0, right_top_distance.x),
                Float::max(0.0, right_top_distance.y),
            );

        transforms
            .get_mut(camera_id)
            .unwrap()
            .set_translation(Vector3::new(
                camera_translation.x,
                camera_translation.y,
                1.0,
            ));
    }
}
