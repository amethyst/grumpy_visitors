pub mod graphics;

use amethyst::{
    assets::{Handle, Prefab},
    ecs::World,
    renderer::{palette::LinSrgba, SpriteSheet},
    ui::FontHandle,
};

use std::{collections::HashMap, time::Duration};

use ha_animation_prefabs::GameSpriteAnimationPrefab;
use ha_core::math::Vector2;

use crate::{
    actions::mob::MobAttackType,
    ecs::resources::graphics::EntityGraphics,
    utils::graphic_helpers::{create_color_material, create_mesh, generate_circle_vertices},
};

pub const HEALTH_UI_SCREEN_PADDING: f32 = 40.0;

pub struct GameTime {
    pub level_started_at: Duration,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            level_started_at: Duration::new(0, 0),
        }
    }
}

#[derive(Clone)]
pub struct AssetsHandles {
    pub hero_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub landscape: Handle<SpriteSheet>,
    pub ui_font: FontHandle,
}

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack_damage: f32,
    pub attack_type: MobAttackType,
    pub graphics: EntityGraphics,
    pub radius: f32,
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

pub struct GameLevelState {
    pub dimensions: Vector2,
    pub is_over: bool,
    pub spawn_level: usize,
    pub spawn_level_started: Duration,
    pub last_borderline_spawn: Duration,
    pub last_random_spawn: Duration,
}

impl GameLevelState {
    pub fn dimensions_half_size(&self) -> Vector2 {
        self.dimensions / 2.0
    }
}

impl Default for GameLevelState {
    fn default() -> Self {
        Self {
            dimensions: Vector2::new(4096.0, 4096.0),
            is_over: false,
            spawn_level: 1,
            spawn_level_started: Duration::new(0, 0),
            last_borderline_spawn: Duration::new(0, 0),
            last_random_spawn: Duration::new(0, 0),
        }
    }
}

pub enum GameEngineState {
    Loading,
    Menu,
    Playing,
}
