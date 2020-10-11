use amethyst::{
    core::math::{Matrix4, Point2, Point3},
    renderer::Camera,
    window::ScreenDimensions,
};

pub fn screen_to_world_from_global_matrix(
    camera: &Camera,
    screen_position: Point2<f32>,
    camera_global_matrix: &Matrix4<f32>,
    screen_dimensions: &ScreenDimensions,
) -> Point3<f32> {
    let screen_x = 2.0 * screen_position.x / screen_dimensions.width() - 1.0;
    let screen_y = 2.0 * screen_position.y / screen_dimensions.height() - 1.0;
    let screen_point = Point3::new(screen_x, screen_y, 0.0).to_homogeneous();
    let vector = camera_global_matrix
        * camera
            .matrix
            .try_inverse()
            .expect("Camera projection matrix is not invertible")
        * screen_point;
    Point3::from_homogeneous(vector).expect("Vector is not homogeneous")
}
