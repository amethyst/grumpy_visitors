use amethyst::ecs::World;
#[cfg(feature = "client")]
use amethyst::renderer::palette::LinSrgba;

use std::collections::HashMap;

#[cfg(feature = "client")]
use ha_client_shared::{
    ecs::resources::EntityGraphics,
    utils::graphic_helpers::{create_color_material, create_mesh, generate_circle_vertices},
};
#[cfg(not(feature = "client"))]
use ha_core::net::client_message::ClientMessagePayload;
#[cfg(feature = "client")]
use ha_core::net::server_message::ServerMessagePayload;
use ha_core::{actions::mob::MobAttackType, net::ConnectionNetEvent};

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack_damage: f32,
    pub attack_type: MobAttackType,
    #[cfg(feature = "client")]
    pub graphics: EntityGraphics,
    pub radius: f32,
}

pub struct MonsterDefinitions(pub HashMap<String, MonsterDefinition>);

impl MonsterDefinitions {
    #[cfg(feature = "client")]
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
        world.insert(Self(map))
    }

    #[cfg(not(feature = "client"))]
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
                radius: 12.0,
            },
        );
        world.insert(Self(map))
    }
}

#[cfg(feature = "client")]
pub struct ConnectionEvents(pub Vec<ConnectionNetEvent<ServerMessagePayload>>);
#[cfg(not(feature = "client"))]
pub struct ConnectionEvents(pub Vec<ConnectionNetEvent<ClientMessagePayload>>);
