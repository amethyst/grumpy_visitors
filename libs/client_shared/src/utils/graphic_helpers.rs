use amethyst::{
    assets::{Handle, Loader},
    prelude::World,
    renderer::{
        palette::LinSrgba,
        rendy::{
            mesh::{MeshBuilder, Position, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        Material, MaterialDefaults, Mesh,
    },
};

use gv_core::math::Vector3;

pub fn generate_rectangle_vertices(
    left_bottom: Vector3,
    right_top: Vector3,
) -> (Vec<Position>, Vec<TexCoord>, Vec<u16>) {
    (
        vec![
            Position([left_bottom.x, left_bottom.y, left_bottom.z]),
            Position([
                right_top.x,
                left_bottom.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
            Position([
                left_bottom.x,
                right_top.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
            Position([right_top.x, right_top.y, right_top.z]),
        ],
        vec![
            TexCoord([0.0, 0.0]),
            TexCoord([1.0, 0.0]),
            TexCoord([0.0, 1.0]),
            TexCoord([1.0, 1.0]),
        ],
        vec![0, 1, 2, 1, 2, 3],
    )
}

pub fn generate_circle_vertices(radius: f32, resolution: usize) -> (Vec<Position>, Vec<TexCoord>) {
    use std::f32::consts::PI;

    let mut positions = Vec::with_capacity(resolution * 3);
    let mut tex_coords = Vec::with_capacity(resolution * 3);
    let angle_offset = 2.0 * PI / resolution as f32;

    // Helper function to generate the vertex at the specified angle.
    let generate_vertex = |angle: f32| {
        let x = angle.cos();
        let y = angle.sin();
        (Position([x * radius, y * radius, 0.0]), TexCoord([x, y]))
    };

    for index in 0..resolution {
        positions.push(Position([0.0, 0.0, 0.0]));
        tex_coords.push(TexCoord([0.0, 0.0]));

        let (position, tex_coord) = generate_vertex(angle_offset * index as f32);
        positions.push(position);
        tex_coords.push(tex_coord);

        let (position, tex_coord) = generate_vertex(angle_offset * (index + 1) as f32);
        positions.push(position);
        tex_coords.push(tex_coord);
    }

    (positions, tex_coords)
}

pub fn create_mesh(
    world: &World,
    positions: Vec<Position>,
    tex_coords: Vec<TexCoord>,
) -> Handle<Mesh> {
    let loader = world.fetch::<Loader>();
    loader.load_from_data(
        MeshBuilder::new()
            .with_vertices(positions)
            .with_vertices(tex_coords)
            .into(),
        (),
        &world.fetch(),
    )
}

pub fn create_color_material(world: &World, colour: LinSrgba) -> Handle<Material> {
    let mat_defaults = world.fetch::<MaterialDefaults>();
    let loader = world.fetch::<Loader>();

    let albedo = loader.load_from_data(load_from_linear_rgba(colour).into(), (), &world.fetch());

    loader.load_from_data(
        Material {
            albedo,
            ..mat_defaults.0.clone()
        },
        (),
        &world.fetch(),
    )
}
