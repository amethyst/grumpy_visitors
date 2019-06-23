use amethyst::core::Float;

pub type Vector2 = amethyst::core::math::Vector2<Float>;
pub type Vector3 = amethyst::core::math::Vector3<Float>;

pub trait ZeroVector {
    fn zero() -> Self;
}

impl ZeroVector for Vector2 {
    fn zero() -> Self {
        Vector2::new(0.0.into(), 0.0.into())
    }
}

impl ZeroVector for Vector3 {
    fn zero() -> Self {
        Vector3::new(0.0.into(), 0.0.into(), 0.0.into())
    }
}
