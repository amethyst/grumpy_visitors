use amethyst::{
    assets::{Handle, Loader, Prefab},
    core::{Float, Transform},
    ecs::{world::EntityResBuilder, Entity, WriteStorage},
    prelude::{Builder, World},
    renderer::{
        palette::LinSrgba,
        rendy::{
            mesh::{MeshBuilder, Position, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        Material, MaterialDefaults, Mesh, SpriteRender, SpriteSheet,
    },
    ui::{Anchor, FontHandle, UiText, UiTransform},
    utils::tag::Tag,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{
    components::*,
    data_resources::{EntityGraphics, GameScene},
    models::{
        common::MonsterDefinition,
        mob_actions::{MobAction, MobActionType},
    },
    tags::*,
    Vector2, Vector3, ZeroVector,
};

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
        .build()
}

pub fn create_monster(
    position: Vector2,
    action: MobAction,
    monster_definition: &MonsterDefinition,
    entity_builder: EntityResBuilder,
    transforms: &mut WriteStorage<Transform>,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    materials: &mut WriteStorage<Handle<Material>>,
    world_positions: &mut WriteStorage<WorldPosition>,
    monsters: &mut WriteStorage<Monster>,
) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(position.x, position.y, 11.0);
    let destination = if let MobActionType::Move(destination) = action.action_type {
        destination
    } else {
        Vector2::zero()
    };

    let MonsterDefinition {
        name,
        base_health,
        base_speed: _base_speed,
        base_attack: _base_attack,
        graphics: EntityGraphics { mesh, material },
        radius,
    } = monster_definition.clone();
    entity_builder
        .with(mesh, meshes)
        .with(material, materials)
        .with(transform, transforms)
        .with(WorldPosition::new(position), world_positions)
        .with(
            Monster {
                health: base_health,
                destination,
                velocity: Vector2::zero(),
                action,
                name,
                radius,
            },
            monsters,
        )
        .build();
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

pub fn create_menu_screen(world: &mut World, font_handle: FontHandle) {
    let some_big_number = Float::from_f32(10000.0);
    let (bg_positions, bg_tex_coords) = generate_rectangle_vertices(
        Vector3::new(-some_big_number, -some_big_number, 0.9.into()),
        Vector3::new(some_big_number, some_big_number, 0.9.into()),
    );
    let mesh = create_mesh(world, bg_positions, bg_tex_coords);
    let color = LinSrgba::new(0.1, 0.1, 0.1, 1.0);
    let material = create_color_material(world, color);
    let transform = Transform::default();
    world
        .create_entity()
        .with(Tag::<UiBackground>::default())
        .with(mesh)
        .with(material)
        .with(transform)
        .build();

    let ui_transform = UiTransform::new(
        "ui_loading".to_owned(),
        Anchor::BottomMiddle,
        Anchor::Middle,
        0.0,
        100.0,
        1.0,
        125.0,
        75.0,
    );
    let mut ui_text = UiText::new(
        font_handle,
        "Loading...".to_owned(),
        [0.9, 0.9, 0.9, 1.0],
        38.0,
    );
    ui_text.align = Anchor::MiddleLeft;
    world
        .create_entity()
        .with(ui_transform)
        .with(ui_text)
        .build();
}

pub fn create_debug_scene_border(world: &mut World) {
    let border_width = Float::from_f32(3.0);

    let screen_dimensions = world.read_resource::<GameScene>().dimensions;
    let half_screen_width = screen_dimensions.x / Float::from_f32(2.0);
    let half_screen_height = screen_dimensions.y / Float::from_f32(2.0);

    let generate_rectangle = |positions: &mut Vec<Position>,
                              tex_coords: &mut Vec<TexCoord>,
                              left_bottom: Vector2,
                              right_top: Vector2| {
        positions.push(Position([
            left_bottom.x.as_f32(),
            right_top.y.as_f32(),
            0.0,
        ]));
        tex_coords.push(TexCoord([0.0, 1.0]));
        positions.push(Position([
            left_bottom.x.as_f32(),
            left_bottom.y.as_f32(),
            0.0,
        ]));
        tex_coords.push(TexCoord([0.0, 0.0]));
        positions.push(Position([
            right_top.x.as_f32(),
            left_bottom.y.as_f32(),
            0.0,
        ]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([
            right_top.x.as_f32(),
            left_bottom.y.as_f32(),
            0.0,
        ]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([right_top.x.as_f32(), right_top.y.as_f32(), 0.0]));
        tex_coords.push(TexCoord([1.0, 1.0]));
        positions.push(Position([
            left_bottom.x.as_f32(),
            right_top.y.as_f32(),
            0.0,
        ]));
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

pub fn generate_rectangle_vertices(
    left_bottom: Vector3,
    right_top: Vector3,
) -> (Vec<Position>, Vec<TexCoord>) {
    (
        vec![
            Position([
                left_bottom.x.as_f32(),
                right_top.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z - left_bottom.z).as_f32() / 2.0,
            ]),
            Position([
                left_bottom.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32(),
            ]),
            Position([
                right_top.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z - left_bottom.z).as_f32() / 2.0,
            ]),
            Position([
                right_top.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z.as_f32() - left_bottom.z.as_f32()) / 2.0,
            ]),
            Position([
                right_top.x.as_f32(),
                right_top.y.as_f32(),
                right_top.z.as_f32(),
            ]),
            Position([
                left_bottom.x.as_f32(),
                right_top.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z.as_f32() - left_bottom.z.as_f32()) / 2.0,
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
