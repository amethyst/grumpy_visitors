use amethyst::ecs::Entity;
use rand::Rng;

use crate::math::Vector2;

#[derive(Debug)]
pub enum MobAction {
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

#[derive(Clone, Debug)]
pub enum MobAttackType {
    #[allow(dead_code)]
    Melee,
    SlowMelee {
        cooldown: f32,
    },
    #[allow(dead_code)]
    Range,
}

impl MobAttackType {
    pub fn randomize_params(&self, factor: f32) -> Self {
        let mut rng = rand::thread_rng();
        match self {
            MobAttackType::SlowMelee { cooldown } => {
                let cooldown = rng.gen_range(cooldown * (1.0 - factor), cooldown * (1.0 + factor));
                MobAttackType::SlowMelee { cooldown }
            }
            other => other.clone(),
        }
    }
}
