use amethyst::renderer::{Material, MeshHandle};

#[derive(Clone)]
pub struct MissileGraphics {
    pub material: Material,
    pub mesh: MeshHandle,
}
