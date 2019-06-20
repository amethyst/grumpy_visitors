use amethyst::{
    core::{
        math::{convert, Matrix4, Point2, Point3},
        Float, Parent, Transform,
    },
    ecs::{world::Builder, Entity, World},
    renderer::{camera::Projection, Camera},
    window::ScreenDimensions,
};

use crate::Vector3;

pub fn screen_to_world(
    camera: &Camera,
    screen_position: Point2<f32>,
    camera_transform: &Transform,
    screen_dimensions: &ScreenDimensions,
) -> Point3<Float> {
    let screen_x = 2.0 * screen_position.x / screen_dimensions.width() - 1.0;
    let screen_y = 2.0 * screen_position.y / screen_dimensions.height() - 1.0;
    let screen_point = Point3::new(screen_x.into(), screen_y.into(), 0.0.into()).to_homogeneous();
    let vector = camera_transform.global_matrix()
        * convert::<_, Matrix4<Float>>(
            camera
                .as_matrix()
                .try_inverse()
                .expect("Camera projection matrix is not invertible"),
        )
        * screen_point;
    Point3::from_homogeneous(vector).expect("Vector is not homogeneous")
}

pub fn screen_to_world_from_global_matrix(
    camera: &Camera,
    screen_position: Point2<f32>,
    camera_global_matrix: &Matrix4<Float>,
    screen_dimensions: &ScreenDimensions,
) -> Point3<Float> {
    let screen_x = 2.0 * screen_position.x / screen_dimensions.width() - 1.0;
    let screen_y = 2.0 * screen_position.y / screen_dimensions.height() - 1.0;
    let screen_point = Point3::new(screen_x.into(), screen_y.into(), 0.0.into()).to_homogeneous();
    let vector = camera_global_matrix
        * convert::<_, Matrix4<Float>>(
            camera
                .as_matrix()
                .try_inverse()
                .expect("Camera projection matrix is not invertible"),
        )
        * screen_point;
    Point3::from_homogeneous(vector).expect("Vector is not homogeneous")
}

pub fn initialise_camera(world: &mut World, player: Entity) {
    let (width, height) = {
        let screen_dimensions = world.read_resource::<ScreenDimensions>();
        (screen_dimensions.width(), screen_dimensions.height())
    };
    let transform = {
        let mut transform = Transform::default();
        transform.set_translation_z(Float::from(100.0));
        transform
    };

    world
        .create_entity()
        .with(Camera::standard_2d(width, height))
        .with(transform)
        .with(Parent::new(player))
        .build();
}
