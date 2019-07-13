use amethyst::{
    assets::{Handle, Loader, Prefab},
    core::Transform,
    ecs::Entity,
    prelude::{Builder, World},
    renderer::{
        palette::LinSrgba,
        rendy::{
            mesh::{MeshBuilder, Position, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        Material, MaterialDefaults, Mesh, SpriteRender, SpriteSheet,
    },
    utils::tag::Tag,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{components::*, data_resources::GameScene, tags::*, Vector2, Vector3, ZeroVector};

pub fn create_player(
    world: &mut World,
    prefab_handle: Handle<Prefab<GameSpriteAnimationPrefab>>,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_z(22.0);

    world
        .create_entity()
        .with(transform)
        .with(prefab_handle)
        .with(PlayerActions::new())
        .with(WorldPosition::new(Vector2::zero()))
        .with(Player::new())
        .with(DamageHistory::new())
        .build()
}

pub fn create_landscape(world: &mut World, landscape_texture_handle: Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    transform.set_translation_z(-1.0);

    world
        .create_entity()
        .with(Tag::<Landscape>::default())
        .with(transform)
        .with(SpriteRender {
            sprite_sheet: landscape_texture_handle.clone(),
            sprite_number: 0,
        })
        .build();
}

pub fn create_debug_scene_border(world: &mut World) {
    let border_width = 3.0;

    let screen_dimensions = world.read_resource::<GameScene>().dimensions;
    let half_screen_width = screen_dimensions.x / 2.0;
    let half_screen_height = screen_dimensions.y / 2.0;

    let generate_rectangle = |positions: &mut Vec<Position>,
                              tex_coords: &mut Vec<TexCoord>,
                              left_bottom: Vector2,
                              right_top: Vector2| {
        positions.push(Position([left_bottom.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 1.0]));
        positions.push(Position([left_bottom.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 0.0]));
        positions.push(Position([right_top.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([right_top.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([right_top.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 1.0]));
        positions.push(Position([left_bottom.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 1.0]));
    };

    let mut positions = Vec::with_capacity(24);
    let mut tex_coords = Vec::with_capacity(24);
    // Top.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, half_screen_height - border_width),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Right.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(half_screen_width - border_width, -half_screen_height),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Bottom.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(half_screen_width, -half_screen_height + border_width),
    );
    // Left.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(-half_screen_width + border_width, half_screen_height),
    );

    let mesh = create_mesh(world, positions, tex_coords);
    let color = LinSrgba::new(0.0, 0.0, 1.0, 1.0);
    let material = create_color_material(world, color);
    let transform = Transform::default();

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(transform)
        .build();
}

#[allow(dead_code)]
pub fn generate_rectangle_vertices(
    left_bottom: Vector3,
    right_top: Vector3,
) -> (Vec<Position>, Vec<TexCoord>) {
    (
        vec![
            Position([
                left_bottom.x,
                right_top.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
            Position([left_bottom.x, left_bottom.y, left_bottom.z]),
            Position([
                right_top.x,
                left_bottom.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
            Position([
                right_top.x,
                left_bottom.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
            Position([right_top.x, right_top.y, right_top.z]),
            Position([
                left_bottom.x,
                right_top.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ]),
        ],
        vec![
            TexCoord([0.0, 1.0]),
            TexCoord([0.0, 0.0]),
            TexCoord([1.0, 0.0]),
            TexCoord([1.0, 0.0]),
            TexCoord([1.0, 1.0]),
            TexCoord([0.0, 1.0]),
        ],
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
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(
        MeshBuilder::new()
            .with_vertices(positions)
            .with_vertices(tex_coords)
            .into(),
        (),
        &world.read_resource(),
    )
}

pub fn create_color_material(world: &World, colour: LinSrgba) -> Handle<Material> {
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let albedo = loader.load_from_data(
        load_from_linear_rgba(colour).into(),
        (),
        &world.read_resource(),
    );

    loader.load_from_data(
        Material {
            albedo,
            ..mat_defaults.0.clone()
        },
        (),
        &world.read_resource(),
    )
}
