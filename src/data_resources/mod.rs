use amethyst::{
    prelude::World,
    renderer::{Material, MeshHandle},
};

use std::collections::HashMap;

use crate::{
    factories::{create_color_material, create_mesh, generate_circle_vertices},
    models::MonsterDefinition,
};

#[derive(Clone)]
pub struct MissileGraphics(pub EntityGraphics);

impl MissileGraphics {
    pub fn register(world: &mut World) {
        let mesh = create_mesh(world, generate_circle_vertices(5.0, 64));
        let material = create_color_material(world, [1.0, 1.0, 1.0, 1.0]);
        world.add_resource(MissileGraphics(EntityGraphics { mesh, material }));
    }
}

#[derive(Clone)]
pub struct EntityGraphics {
    pub material: Material,
    pub mesh: MeshHandle,
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
                base_speed: 400.0,
                base_attack: 20.0,
                graphics: {
                    let color = [0.3, 0.3, 0.3, 1.0];
                    let material = create_color_material(world, color);
                    let mesh = create_mesh(world, generate_circle_vertices(12.0, 64));

                    EntityGraphics { material, mesh }
                },
            },
        );
        world.add_resource(Self(map))
    }
}
