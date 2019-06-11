use amethyst::ecs::Entity;

use std::time::Duration;

use crate::{data_resources::EntityGraphics, Vector2};

pub enum GameState {
    Loading,
    Playing,
}

pub struct SpawnActions(pub Vec<SpawnAction>);

pub struct SpawnAction {
    pub monsters: Count<String>,
    pub spawn_type: SpawnType,
}

pub struct Count<T> {
    pub entity: T,
    pub num: u8,
}

pub enum SpawnType {
    Random,
    Borderline,
}

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack: f32,
    pub graphics: EntityGraphics,
}

#[derive(Debug)]
pub struct MonsterAction {
    pub started_at: Duration,
    pub action_type: MonsterActionType,
}

impl MonsterAction {
    pub fn idle(started_at: Duration) -> Self {
        Self {
            started_at,
            action_type: MonsterActionType::Idle,
        }
    }
}

#[derive(Debug)]
pub enum MonsterActionType {
    /// Just spawned.
    Idle,
    /// Moving to the specified position.
    Move(Vector2),
    /// Chasing an entity with the specified id.
    Chase(Entity),
    #[allow(dead_code)]
    Attack(AttackAction),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AttackAction {
    /// Entity id.
    pub target: Entity,
    pub attack_type: AttackType,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AttackType {
    Melee,
    SlowMelee,
    Range,
}
