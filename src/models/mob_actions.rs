use amethyst::ecs::Entity;

use std::time::Duration;

use crate::Vector2;

#[derive(Debug)]
pub struct MobAction {
    pub started_at: Duration,
    pub action_type: MobActionType,
}

impl MobAction {
    pub fn idle(started_at: Duration) -> Self {
        Self {
            started_at,
            action_type: MobActionType::Idle,
        }
    }
}

#[derive(Debug)]
pub enum MobActionType {
    /// Just spawned.
    Idle,
    /// Moving to the specified position.
    Move(Vector2),
    /// Chasing an entity with the specified id.
    Chase(Entity),
    /// Attacking a target.
    Attack(MobAttackAction),
}

#[derive(Debug)]
pub struct MobAttackAction {
    /// Entity id.
    pub target: Entity,
    pub attack_type: MobAttackType,
}

#[derive(Clone, Copy, Debug)]
pub enum MobAttackType {
    #[allow(dead_code)]
    Melee,
    SlowMelee,
    #[allow(dead_code)]
    Range,
}
