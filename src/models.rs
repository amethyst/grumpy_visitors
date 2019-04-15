use amethyst::ecs::Entity;

use std::time::Instant;

use crate::{
    data_resources::EntityGraphics,
    Vector2,
};

pub struct SpawnActions(pub Vec<SpawnAction>);

pub struct SpawnAction {
    pub monsters: Count<String>,
}

pub struct Count<T> {
    pub entity: T,
    pub num: u8,
}

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack: f32,
    pub graphics: EntityGraphics,
}

pub struct MonsterAction {
    pub started_at: Instant,
    pub action_type: MonsterActionType,
}

impl MonsterAction {
    pub fn idle() -> Self {
        Self {
            started_at: Instant::now(),
            action_type: MonsterActionType::Idle,
        }
    }
}

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
pub struct AttackAction {
    /// Entity id.
    pub target: Entity,
    pub attack_type: AttackType,
}

#[allow(dead_code)]
pub enum AttackType {
    Melee,
    SlowMelee,
    Range,
}
