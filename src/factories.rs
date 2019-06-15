use amethyst::{
    assets::{Handle, Loader, Prefab},
    core::{Float, Transform},
    ecs::{world::EntityResBuilder, Entity, WriteStorage},
    prelude::{Builder, World},
    renderer::{
        palette::LinSrgba,
        rendy::{
            mesh::{MeshBuilder, PosTex, Position, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        Material, MaterialDefaults, Mesh, Texture,
    },
    ui::{Anchor, FontHandle, UiText, UiTransform},
    utils::tag::Tag,
};

use std::time::Instant;

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{
    components::*,
    data_resources::{EntityGraphics, GameScene},
    models::{MonsterAction, MonsterActionType, MonsterDefinition},
    tags::*,
    Vector2, Vector3,
};

pub fn create_missile(
    position: Vector2,
    direction: Vector2,
    time_spawned: Instant,
    entity_builder: EntityResBuilder,
    missile_graphic: EntityGraphics,
    transforms: &mut WriteStorage<Transform>,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    materials: &mut WriteStorage<Handle<Material>>,
    world_positions: &mut WriteStorage<WorldPosition>,
    missiles: &mut WriteStorage<Missile>,
) {
    let EntityGraphics { mesh, material } = missile_graphic;
    let mut transform = Transform::default();
    transform.set_translation_xyz(position.x, position.y, 0.0);

    entity_builder
        .with(mesh, meshes)
        .with(material, materials)
        .with(transform, transforms)
        .with(WorldPosition::new(position), world_positions)
        .with(Missile::new(direction - position, time_spawned), missiles)
        .build();
}

pub fn create_player(
    world: &mut World,
    prefab_handle: Handle<Prefab<GameSpriteAnimationPrefab>>,
) -> Entity {
    let transform = Transform::default();

    world
        .create_entity()
        .with(transform)
        .with(prefab_handle)
        .with(WorldPosition::new(Vector2::new(0.0.into(), 0.0.into())))
        .with(Player::new())
        .build()
}

pub fn create_monster(
    position: Vector2,
    action: MonsterAction,
    monster_definition: &MonsterDefinition,
    entity_builder: EntityResBuilder,
    transforms: &mut WriteStorage<Transform>,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    materials: &mut WriteStorage<Handle<Material>>,
    world_positions: &mut WriteStorage<WorldPosition>,
    monsters: &mut WriteStorage<Monster>,
) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(position.x, position.y, 0.0);
    let destination = if let MonsterActionType::Move(destination) = action.action_type {
        destination
    } else {
        Vector2::new(0.0.into(), 0.0.into())
    };

    let MonsterDefinition {
        name,
        base_health,
        base_speed: _base_speed,
        base_attack: _base_attack,
        graphics: EntityGraphics { mesh, material },
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
                name,
                action,
            },
            monsters,
        )
        .build();
}

pub fn create_landscape(world: &mut World, landscape_texture_handle: Handle<Texture>) {
    let mut transform = Transform::default();
    transform.set_translation_z(-1.0);

    world
        .create_entity()
        .with(transform)
        .with(landscape_texture_handle)
        .build();
}

pub fn create_menu_screen(world: &mut World, font_handle: FontHandle) {
    let some_big_number = Float::from_f32(10000.0);
    let ui_background_vertices = generate_rectangle_vertices(
        Vector3::new(-some_big_number, -some_big_number, 0.9.into()),
        Vector3::new(some_big_number, some_big_number, 0.9.into()),
    );
    let mesh = create_mesh(world, ui_background_vertices);
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
        Anchor::MiddleLeft,
        0.0,
        100.0,
        1.0,
        200.0,
        100.0,
    );
    let ui_text = UiText::new(
        font_handle,
        "Loading...".to_owned(),
        [0.9, 0.9, 0.9, 1.0],
        38.0,
    );
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

    let generate_rectangle =
        |vertices: &mut Vec<PosTex>, left_bottom: Vector2, right_top: Vector2| {
            vertices.push(PosTex {
                position: Position([left_bottom.x.as_f32(), right_top.y.as_f32(), 0.0]),
                tex_coord: TexCoord([0.0, 1.0]),
            });
            vertices.push(PosTex {
                position: Position([left_bottom.x.as_f32(), left_bottom.y.as_f32(), 0.0]),
                tex_coord: TexCoord([0.0, 0.0]),
            });
            vertices.push(PosTex {
                position: Position([right_top.x.as_f32(), left_bottom.y.as_f32(), 0.0]),
                tex_coord: TexCoord([1.0, 0.0]),
            });
            vertices.push(PosTex {
                position: Position([right_top.x.as_f32(), left_bottom.y.as_f32(), 0.0]),
                tex_coord: TexCoord([1.0, 0.0]),
            });
            vertices.push(PosTex {
                position: Position([right_top.x.as_f32(), right_top.y.as_f32(), 0.0]),
                tex_coord: TexCoord([1.0, 1.0]),
            });
            vertices.push(PosTex {
                position: Position([left_bottom.x.as_f32(), right_top.y.as_f32(), 0.0]),
                tex_coord: TexCoord([0.0, 1.0]),
            });
        };

    let mut vertices = Vec::with_capacity(24);
    // Top.
    generate_rectangle(
        &mut vertices,
        Vector2::new(-half_screen_width, half_screen_height - border_width),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Right.
    generate_rectangle(
        &mut vertices,
        Vector2::new(half_screen_width - border_width, -half_screen_height),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Bottom.
    generate_rectangle(
        &mut vertices,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(half_screen_width, -half_screen_height + border_width),
    );
    // Left.
    generate_rectangle(
        &mut vertices,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(-half_screen_width + border_width, half_screen_height),
    );

    let mesh = create_mesh(world, vertices);
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

pub fn generate_rectangle_vertices(left_bottom: Vector3, right_top: Vector3) -> Vec<PosTex> {
    vec![
        PosTex {
            position: Position([
                left_bottom.x.as_f32(),
                right_top.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z - left_bottom.z).as_f32() / 2.0,
            ]),
            tex_coord: TexCoord([0.0, 1.0]),
        },
        PosTex {
            position: Position([
                left_bottom.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32(),
            ]),
            tex_coord: TexCoord([0.0, 0.0]),
        },
        PosTex {
            position: Position([
                right_top.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z - left_bottom.z).as_f32() / 2.0,
            ]),
            tex_coord: TexCoord([1.0, 0.0]),
        },
        PosTex {
            position: Position([
                right_top.x.as_f32(),
                left_bottom.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z.as_f32() - left_bottom.z.as_f32()) / 2.0,
            ]),
            tex_coord: TexCoord([1.0, 0.0]),
        },
        PosTex {
            position: Position([
                right_top.x.as_f32(),
                right_top.y.as_f32(),
                right_top.z.as_f32(),
            ]),
            tex_coord: TexCoord([1.0, 1.0]),
        },
        PosTex {
            position: Position([
                left_bottom.x.as_f32(),
                right_top.y.as_f32(),
                left_bottom.z.as_f32() + (right_top.z.as_f32() - left_bottom.z.as_f32()) / 2.0,
            ]),
            tex_coord: TexCoord([0.0, 1.0]),
        },
    ]
}

pub fn generate_circle_vertices(radius: f32, resolution: usize) -> Vec<PosTex> {
    use std::f32::consts::PI;

    let mut vertices = Vec::with_capacity(resolution * 3);
    let angle_offset = 2.0 * PI / resolution as f32;

    // Helper function to generate the vertex at the specified angle.
    let generate_vertex = |angle: f32| {
        let x = angle.cos();
        let y = angle.sin();
        PosTex {
            position: Position([x * radius, y * radius, 0.0]),
            tex_coord: TexCoord([x, y]),
        }
    };

    for index in 0..resolution {
        vertices.push(PosTex {
            position: Position([0.0, 0.0, 0.0]),
            tex_coord: TexCoord([0.0, 0.0]),
        });

        vertices.push(generate_vertex(angle_offset * index as f32));
        vertices.push(generate_vertex(angle_offset * (index + 1) as f32));
    }

    vertices
}

pub fn create_mesh(world: &World, vertices: Vec<PosTex>) -> Handle<Mesh> {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(
        MeshBuilder::new().with_vertices(vertices).into(),
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
