use amethyst::{
    animation::AnimationSetPrefab,
    assets::{PrefabData, ProgressCounter},
    derive::PrefabData,
    ecs::Entity,
    error::Error,
    renderer::{SpriteRender, SpriteScenePrefab},
};
use serde_derive::{Deserialize, Serialize};

/// Animation ids used in a AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    Walk,
}

/// Loading data for one entity
#[derive(Debug, Clone, Deserialize, Serialize, PrefabData)]
pub struct GameSpritePrefab {
    /// Information for rendering a scene with sprites
    pub sprite_scene: SpriteScenePrefab,
    /// Аll animations that can be run on the entity
    pub animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}
