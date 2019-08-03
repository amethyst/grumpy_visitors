pub type Vector2 = amethyst::core::math::Vector2<f32>;
pub type Vector3 = amethyst::core::math::Vector3<f32>;

pub trait ZeroVector {
    fn zero() -> Self;
}

impl ZeroVector for Vector2 {
    fn zero() -> Self {
        Vector2::new(0.0, 0.0)
    }
}

impl ZeroVector for Vector3 {
    fn zero() -> Self {
        Vector3::new(0.0, 0.0, 0.0)
    }
}
