#![allow(clippy::type_repetition_in_bounds)]

use amethyst::{
    assets::{Handle, Prefab},
    core::{Parent, Transform},
    ecs::{prelude::World, Entities, Entity, ReadExpect, WriteStorage},
    renderer::Camera,
    shred::{ResourceId, SystemData},
    window::ScreenDimensions,
};

use gv_animation_prefabs::GameSpriteAnimationPrefab;
use gv_core::{ecs::components::ClientPlayerActions, math::Vector2};

use crate::ecs::{
    components::HealthUiGraphics,
    resources::{AssetHandles, HEALTH_UI_SCREEN_PADDING},
};

#[derive(SystemData)]
pub struct CameraFactory<'s> {
    entities: Entities<'s>,
    screen_dimensions: ReadExpect<'s, ScreenDimensions>,
    cameras: WriteStorage<'s, Camera>,
    transforms: WriteStorage<'s, Transform>,
    parents: WriteStorage<'s, Parent>,
}

impl<'s> CameraFactory<'s> {
    pub fn create(&mut self, player: Entity) {
        let (width, height) = (
            self.screen_dimensions.width(),
            self.screen_dimensions.height(),
        );
        let transform = {
            let mut transform = Transform::default();
            transform.set_translation_z(100.0);
            transform
        };

        self.entities
            .build_entity()
            .with(Camera::standard_2d(width, height), &mut self.cameras)
            .with(transform, &mut self.transforms)
            .with(Parent::new(player), &mut self.parents)
            .build();
    }
}

#[derive(SystemData)]
pub struct PlayerClientFactory<'s> {
    asset_handles: ReadExpect<'s, AssetHandles>,
    screen_dimensions: ReadExpect<'s, ScreenDimensions>,
    sprite_animation_handles: WriteStorage<'s, Handle<Prefab<GameSpriteAnimationPrefab>>>,
    health_ui_graphics: WriteStorage<'s, HealthUiGraphics>,
    client_player_actions: WriteStorage<'s, ClientPlayerActions>,
}

impl<'s> PlayerClientFactory<'s> {
    pub fn create(&mut self, player_entity: Entity, is_controllable: bool) {
        let AssetHandles { mage_prefab, .. } = self.asset_handles.clone();

        let mut transform = Transform::default();
        transform.set_translation_z(10.0);

        let (half_screen_width, half_screen_height) = (
            self.screen_dimensions.width() / 2.0,
            self.screen_dimensions.height() / 2.0,
        );

        self.sprite_animation_handles
            .insert(player_entity, mage_prefab)
            .expect("Expected to insert a HeroPrefab");
        if is_controllable {
            self.health_ui_graphics
                .insert(
                    player_entity,
                    HealthUiGraphics {
                        screen_position: Vector2::new(
                            -half_screen_width + HEALTH_UI_SCREEN_PADDING,
                            -half_screen_height + HEALTH_UI_SCREEN_PADDING,
                        ),
                        scale_ratio: 1.0,
                        health: 1.0,
                    },
                )
                .expect("Expected to insert a HealthUiGraphics component");

            self.client_player_actions
                .insert(player_entity, ClientPlayerActions::default())
                .expect("Expected to insert a ClientPlayerActions component");
        }
    }
}
