use amethyst::{
    assets::{Handle, Loader, Prefab},
    core::Transform,
    ecs::{world::EntityResBuilder, Entity, WriteStorage},
    prelude::{Builder, World},
    renderer::{Material, MaterialDefaults, MeshHandle, PosTex, TextureHandle},
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
    meshes: &mut WriteStorage<MeshHandle>,
    materials: &mut WriteStorage<Material>,
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
        .with(WorldPosition::new(Vector2::new(0.0, 0.0)))
        .with(Player::new())
        .build()
}

pub fn create_monster(
    position: Vector2,
    action: MonsterAction,
    monster_definition: &MonsterDefinition,
    entity_builder: EntityResBuilder,
    transforms: &mut WriteStorage<Transform>,
    meshes: &mut WriteStorage<MeshHandle>,
    materials: &mut WriteStorage<Material>,
    world_positions: &mut WriteStorage<WorldPosition>,
    monsters: &mut WriteStorage<Monster>,
) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(position.x, position.y, 0.0);
    let destination = if let MonsterActionType::Move(destination) = action.action_type {
        destination
    } else {
        Vector2::new(0.0, 0.0)
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

pub fn create_landscape(world: &mut World, landscape_texture_handle: TextureHandle) {
    let mut transform = Transform::default();
    transform.set_translation_z(-1.0);

    world
        .create_entity()
        .with(transform)
        .with(landscape_texture_handle)
        .build();
}

pub fn create_menu_screen(world: &mut World, font_handle: FontHandle) {
    let some_big_number = 10000.0;
    let ui_background_vertices = generate_rectangle_vertices(
        Vector3::new(-some_big_number, -some_big_number, 0.9),
        Vector3::new(some_big_number, some_big_number, 0.9),
    );
    let mesh = create_mesh(world, ui_background_vertices);
    let color = [0.1, 0.1, 0.1, 1.0];
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
    let border_width = 3.0;

    let screen_dimensions = world.read_resource::<GameScene>().dimensions;
    let half_screen_width = screen_dimensions.x / 2.0;
    let half_screen_height = screen_dimensions.y / 2.0;

    let generate_rectangle =
        |vertices: &mut Vec<PosTex>, left_bottom: Vector2, right_top: Vector2| {
            vertices.push(PosTex {
                position: Vector3::new(left_bottom.x, right_top.y, 0.0),
                tex_coord: Vector2::new(0.0, 1.0),
            });
            vertices.push(PosTex {
                position: Vector3::new(left_bottom.x, left_bottom.y, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
            });
            vertices.push(PosTex {
                position: Vector3::new(right_top.x, left_bottom.y, 0.0),
                tex_coord: Vector2::new(1.0, 0.0),
            });
            vertices.push(PosTex {
                position: Vector3::new(right_top.x, left_bottom.y, 0.0),
                tex_coord: Vector2::new(1.0, 0.0),
            });
            vertices.push(PosTex {
                position: Vector3::new(right_top.x, right_top.y, 0.0),
                tex_coord: Vector2::new(1.0, 1.0),
            });
            vertices.push(PosTex {
                position: Vector3::new(left_bottom.x, right_top.y, 0.0),
                tex_coord: Vector2::new(0.0, 1.0),
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
    let color = [0.0, 0.0, 1.0, 1.0];
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
            position: Vector3::new(
                left_bottom.x,
                right_top.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ),
            tex_coord: Vector2::new(0.0, 1.0),
        },
        PosTex {
            position: Vector3::new(left_bottom.x, left_bottom.y, left_bottom.z),
            tex_coord: Vector2::new(0.0, 0.0),
        },
        PosTex {
            position: Vector3::new(
                right_top.x,
                left_bottom.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ),
            tex_coord: Vector2::new(1.0, 0.0),
        },
        PosTex {
            position: Vector3::new(
                right_top.x,
                left_bottom.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ),
            tex_coord: Vector2::new(1.0, 0.0),
        },
        PosTex {
            position: Vector3::new(right_top.x, right_top.y, right_top.z),
            tex_coord: Vector2::new(1.0, 1.0),
        },
        PosTex {
            position: Vector3::new(
                left_bottom.x,
                right_top.y,
                left_bottom.z + (right_top.z - left_bottom.z) / 2.0,
            ),
            tex_coord: Vector2::new(0.0, 1.0),
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
            position: Vector3::new(x * radius, y * radius, 0.0),
            tex_coord: Vector2::new(x, y),
        }
    };

    for index in 0..resolution {
        vertices.push(PosTex {
            position: Vector3::new(0.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
        });

        vertices.push(generate_vertex(angle_offset * index as f32));
        vertices.push(generate_vertex(angle_offset * (index + 1) as f32));
    }

    vertices
}

pub fn create_mesh(world: &World, vertices: Vec<PosTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), (), &world.read_resource())
}

pub fn create_color_material(world: &World, colour: [f32; 4]) -> Material {
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let albedo = loader.load_from_data(colour.into(), (), &world.read_resource());

    Material {
        albedo,
        ..mat_defaults.0.clone()
    }
}
