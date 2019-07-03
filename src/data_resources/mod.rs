use amethyst::{
    assets::Handle,
    prelude::World,
    renderer::{palette::LinSrgba, Material, Mesh},
};

use std::collections::HashMap;

use crate::{
    factories::{create_color_material, create_mesh, generate_circle_vertices},
    models::common::MonsterDefinition,
    Vector2,
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
                base_attack: 20.0,
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
