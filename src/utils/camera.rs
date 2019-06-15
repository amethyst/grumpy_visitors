use amethyst::{
    core::{
        math::{convert, Matrix4, Point2, Point3},
        Float, Transform, Parent,
    },
    ecs::{Entity, World, world::Builder},
    renderer::{Camera, camera::Projection},
    window::ScreenDimensions,
};

use crate::Vector3;

pub fn position_from_screen(
    camera: &Camera,
    screen_position: Point2<f32>,
    camera_transform: &Transform,
    screen_dimensions: &ScreenDimensions,
) -> Point3<Float> {
    let screen_x = 2.0 * screen_position.x / screen_dimensions.width() - 1.0;
    let screen_y = 1.0 - 2.0 * screen_position.y / screen_dimensions.height();
    let screen_point = Point3::new(screen_x.into(), screen_y.into(), 0.0.into()).to_homogeneous();
    let vector = camera_transform.matrix()
        * convert::<_, Matrix4<Float>>(
            camera
                .projection()
                .as_matrix()
                .try_inverse()
                .expect("Camera projection matrix is not invertible"),
        )
        * screen_point;
    Point3::from_homogeneous(vector).expect("Vector is not homogeneous")
    //    Point3::new(0.0, 0.0, 0.0)
}

pub fn initialise_camera(world: &mut World, player: Entity) {
    let transform = {
        let screen_dimensions = world.read_resource::<ScreenDimensions>();
        let mut transform = Transform::default();
        transform.set_translation(Vector3::new(
            (-screen_dimensions.width() / 2.0).into(),
            (-screen_dimensions.height() / 2.0).into(),
            1.0.into(),
        ));
        transform
    };

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 1024.0, 0.0, 768.0, -1000.0, 1000.0,
        )))
        .with(transform)
        .with(Parent::new(player))
        .build();
}
