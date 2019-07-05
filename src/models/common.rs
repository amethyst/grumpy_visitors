use amethyst::{
    assets::{Handle, Prefab},
    ecs::Entity,
    renderer::SpriteSheet,
    ui::FontHandle,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use std::time::Duration;

use crate::{data_resources::EntityGraphics, models::mob_actions::MobAttackType, Vector2};

pub enum GameState {
    Loading,
    Playing,
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

#[derive(Clone)]
pub struct AssetsHandles {
    pub hero_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub landscape: Handle<SpriteSheet>,
    pub ui_font: FontHandle,
}

#[derive(Clone)]
pub enum MissileTarget {
    Target(Entity),
    Destination(Vector2),
}

pub struct DamageHistoryEntries {
    pub time: Duration,
    pub entries: Vec<DamageHistoryEntry>,
}

impl DamageHistoryEntries {
    pub fn new(time: Duration) -> Self {
        Self {
            time,
            entries: Vec::new(),
        }
    }
}

pub struct DamageHistoryEntry {
    pub damage: f32,
}
