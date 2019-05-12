use amethyst::{
    animation::AnimationSetPrefab,
    assets::{PrefabData, ProgressCounter},
    core::Named,
    derive::PrefabData,
    ecs::{Entity, WriteStorage},
    error::Error,
    renderer::{SpriteRender, SpriteScenePrefab, Transparent},
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
    #[serde(skip)]
    #[prefab(Component)]
    pub transparent: Transparent,
}
