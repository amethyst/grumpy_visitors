use amethyst::{
    assets::{Handle, Loader},
    prelude::World,
    renderer::{palette::LinSrgba, rendy::mesh::MeshBuilder, Material, Mesh},
};

use std::collections::HashMap;

use crate::{
    factories::{
        create_color_material, create_mesh, generate_circle_vertices, generate_rectangle_vertices,
    },
    models::{common::MonsterDefinition, mob_actions::MobAttackType},
    Vector2, Vector3,
};

pub const HEALTH_UI_SCREEN_PADDING: f32 = 40.0;

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

pub struct MonsterDefinitions(pub HashMap<String, MonsterDefinition>);

impl MonsterDefinitions {
    pub fn register(world: &mut World) {
        let mut map = HashMap::new();
        map.insert(
            "Ghoul".to_owned(),
            MonsterDefinition {
                name: "Ghoul".to_owned(),
                base_health: 100.0,
                base_speed: 180.0,
                base_attack_damage: 15.0,
                attack_type: MobAttackType::SlowMelee { cooldown: 0.75 },
                graphics: {
                    let color = LinSrgba::new(0.21, 0.06, 0.11, 1.0);
                    let material = create_color_material(world, color);
                    let (positions, tex_coords) = generate_circle_vertices(12.0, 64);
                    let mesh = create_mesh(world, positions, tex_coords);
                    EntityGraphics { material, mesh }
                },
                radius: 12.0,
            },
        );
        world.add_resource(Self(map))
    }
}

pub struct GameScene {
    pub dimensions: Vector2,
}

impl GameScene {
    pub fn half_size(&self) -> Vector2 {
        self.dimensions / 2.0
    }
}

impl Default for GameScene {
    fn default() -> Self {
        Self {
            dimensions: Vector2::new(4096.0, 4096.0),
        }
    }
}
