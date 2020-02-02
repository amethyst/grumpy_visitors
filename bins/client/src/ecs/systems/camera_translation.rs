use amethyst::{
    core::{
        math::{Point2, Vector2, Vector3},
        transform::{Parent, Transform},
    },
    ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage},
    renderer::Camera,
    window::ScreenDimensions,
};

use gv_core::ecs::resources::GameLevelState;

use crate::utils::camera;

pub struct CameraTranslationSystem;

impl<'s> System<'s> for CameraTranslationSystem {
    type SystemData = (
        ReadExpect<'s, GameLevelState>,
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
        let relaxed_camera_transform = transforms.get(camera_parent.entity).unwrap().clone();

        let screen_left_bottom = camera::screen_to_world_from_global_matrix(
            &camera,
            Point2::new(0.0, screen_dimensions.height()),
            &relaxed_camera_transform.matrix(),
            &screen_dimensions,
        );
        let screen_left_bottom = Vector2::new(screen_left_bottom.x, screen_left_bottom.y);
        let screen_right_top = camera::screen_to_world_from_global_matrix(
            &camera,
            Point2::new(screen_dimensions.width(), 0.0),
            &relaxed_camera_transform.matrix(),
            &screen_dimensions,
        );
        let screen_right_top = Vector2::new(screen_right_top.x, screen_right_top.y);

        let left_bottom_distance = -screen_left_bottom - game_scene.dimensions_half_size();
        let right_top_distance = screen_right_top - game_scene.dimensions_half_size();

        let camera_translation = Vector2::new(
            num::Float::max(0.0, left_bottom_distance.x),
            num::Float::max(0.0, left_bottom_distance.y),
        ) - Vector2::new(
            num::Float::max(0.0, right_top_distance.x),
            num::Float::max(0.0, right_top_distance.y),
        );

        let camera_transform = transforms.get_mut(camera_id).unwrap();
        camera_transform.set_translation(Vector3::new(
            camera_translation.x,
            camera_translation.y,
            camera_transform.translation().z,
        ));
    }
}
