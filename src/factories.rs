use amethyst::{
    assets::Loader,
    core::Transform,
    ecs::{world::EntityResBuilder, Entity, WriteStorage},
    prelude::{Builder, World},
    renderer::{Material, MaterialDefaults, MeshHandle, PosTex},
};

use std::time::Instant;

use crate::{
    components::*,
    data_resources::EntityGraphics,
    models::{MonsterAction, MonsterDefinition},
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

pub fn create_player(world: &mut World) -> Entity {
    let mesh = create_mesh(world, generate_circle_vertices(15.0, 64));
    let color = [1.0, 1.0, 1.0, 1.0];
    let material = create_color_material(world, color);
    let mut transform = Transform::default();
    transform.set_translation_xyz(500.0, 300.0, 0.0);

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(transform)
        .with(WorldPosition::new(Vector2::new(500.0, 300.0)))
        .with(Player::new())
        .build()
}

pub fn create_monster(
    position: Vector2,
    _direction: Vector2,
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
                velocity: Vector2::zeros(),
                name,
                action: MonsterAction::idle(),
            },
            monsters,
        )
        .build();
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
