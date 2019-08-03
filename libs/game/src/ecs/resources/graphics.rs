use amethyst::{
    assets::{Handle, Loader},
    prelude::World,
    renderer::{palette::LinSrgba, rendy::mesh::MeshBuilder, Material, Mesh},
};

use ha_core::math::Vector3;

use crate::utils::graphic_helpers::{
    create_color_material, create_mesh, generate_circle_vertices, generate_rectangle_vertices,
};

#[derive(Clone)]
pub struct MissileGraphics(pub EntityGraphics);

impl MissileGraphics {
    pub fn register(world: &mut World) {
        let (positions, tex_coords) = generate_circle_vertices(5.0, 64);
        let mesh = create_mesh(world, positions, tex_coords);
        let material = create_color_material(world, LinSrgba::new(1.0, 0.0, 0.0, 1.0));
        world.add_resource(MissileGraphics(EntityGraphics { mesh, material }));
    }
}

#[derive(Clone)]
pub struct HealthUiMesh(pub Handle<Mesh>);

impl HealthUiMesh {
    pub fn register(world: &mut World) {
        let (vertices, tex_coords, indices) = generate_rectangle_vertices(
            Vector3::new(0.0, 0.0, 100.0),
            Vector3::new(180.0, 180.0, 100.0),
        );

        let mesh = {
            let loader = world.read_resource::<Loader>();
            loader.load_from_data(
                MeshBuilder::new()
                    .with_vertices(vertices)
                    .with_vertices(tex_coords)
                    .with_indices(indices)
                    .into(),
                (),
                &world.read_resource(),
            )
        };
        world.add_resource(HealthUiMesh(mesh));
    }
}

#[derive(Clone)]
pub struct EntityGraphics {
    pub material: Handle<Material>,
    pub mesh: Handle<Mesh>,
}
