use amethyst::{
    animation::AnimationSetPrefab,
    assets::{PrefabData, ProgressCounter},
    derive::PrefabData,
    ecs::Entity,
    core::Named,
    error::Error,
    renderer::{SpriteRender, SpriteScenePrefab},
};
use serde_derive::{Deserialize, Serialize};

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    Walk,
}

#[derive(Debug, Clone, Deserialize, Serialize, PrefabData)]
pub struct GameSpriteAnimationPrefab {
    pub name: Named,
    pub sprite_scene: SpriteScenePrefab,
    pub animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}
