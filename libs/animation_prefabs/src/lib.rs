use amethyst::{
    animation::AnimationSetPrefab,
    assets::{PrefabData, ProgressCounter},
    core::Named,
    derive::PrefabData,
    ecs::{Entity, WriteStorage},
    error::Error,
    renderer::{sprite::prefab::SpriteScenePrefab, SpriteRender, Transparent},
};
use serde_derive::{Deserialize, Serialize};

pub const MAGE_TORSO: &str = "mage_torso";
pub const MAGE_LEGS: &str = "mage_legs";
pub const MONSTER_BODY: &str = "monster_body";

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    Walk,
    Attack,
    Death,
    Spell1,
    Spell2,
}

#[derive(Debug, Clone, Deserialize, Serialize, PrefabData)]
pub struct GameSpriteAnimationPrefab {
    pub name_tag: Named,
    pub sprite_scene: SpriteScenePrefab,
    pub animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
    #[serde(skip)]
    #[prefab(Component)]
    pub transparent: Transparent,
}
