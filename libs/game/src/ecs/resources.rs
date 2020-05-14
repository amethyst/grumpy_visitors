use amethyst::ecs::World;

use std::collections::HashMap;

#[cfg(not(feature = "client"))]
use gv_core::net::client_message::ClientMessage;
#[cfg(feature = "client")]
use gv_core::net::server_message::ServerMessage;
use gv_core::{actions::mob::MobAttackType, net::ConnectionNetEvent};

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack_damage: f32,
    pub attack_type: MobAttackType,
    pub collision_radius: f32,
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
                collision_radius: 12.0,
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
                collision_radius: 12.0,
            },
        );
        world.insert(Self(map))
    }
}

#[cfg(feature = "client")]
pub struct ConnectionEvents(pub Vec<ConnectionNetEvent<ServerMessage>>);
#[cfg(not(feature = "client"))]
pub struct ConnectionEvents(pub Vec<ConnectionNetEvent<ClientMessage>>);
