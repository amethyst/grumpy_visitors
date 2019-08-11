#[cfg(feature = "client")]
use amethyst::{
    assets::{Handle, Prefab},
    ecs::ReadExpect,
    renderer::{Material, Mesh, SpriteRender},
    window::ScreenDimensions,
};
use amethyst::{
    core::Transform,
    ecs::{Entities, Entity, WriteStorage},
    utils::tag::Tag,
};
use shred_derive::SystemData;

use std::time::Duration;

#[cfg(feature = "client")]
use ha_animation_prefabs::GameSpriteAnimationPrefab;
#[cfg(feature = "client")]
use ha_client_shared::ecs::{
    components::HealthUiGraphics,
    resources::{AssetHandles, EntityGraphics, MissileGraphics, HEALTH_UI_SCREEN_PADDING},
};
use ha_core::{
    actions::mob::MobAction,
    ecs::{
        components::{damage_history::DamageHistory, missile::*, *},
        tags::*,
    },
    math::{Vector2, ZeroVector},
};

use crate::ecs::resources::MonsterDefinition;

#[derive(SystemData)]
pub struct PlayerFactory<'s> {
    entities: Entities<'s>,
    #[cfg(feature = "client")]
    asset_handles: ReadExpect<'s, AssetHandles>,
    #[cfg(feature = "client")]
    screen_dimensions: ReadExpect<'s, ScreenDimensions>,
    transforms: WriteStorage<'s, Transform>,
    #[cfg(feature = "client")]
    sprite_animation_handles: WriteStorage<'s, Handle<Prefab<GameSpriteAnimationPrefab>>>,
    player_actions: WriteStorage<'s, PlayerActions>,
    world_positions: WriteStorage<'s, WorldPosition>,
    players: WriteStorage<'s, Player>,
    damage_histories: WriteStorage<'s, DamageHistory>,
    #[cfg(feature = "client")]
    health_ui_graphics: WriteStorage<'s, HealthUiGraphics>,
}

impl<'s> PlayerFactory<'s> {
    #[cfg(feature = "client")]
    pub fn create(&mut self) -> Entity {
        let AssetHandles { hero_prefab, .. } = self.asset_handles.clone();

        let mut transform = Transform::default();
        transform.set_translation_z(10.0);

        let (half_screen_width, half_screen_height) = (
            self.screen_dimensions.width() / 2.0,
            self.screen_dimensions.height() / 2.0,
        );

        self.entities
            .build_entity()
            .with(transform, &mut self.transforms)
            .with(hero_prefab, &mut self.sprite_animation_handles)
            .with(PlayerActions::default(), &mut self.player_actions)
            .with(
                WorldPosition::new(Vector2::zero()),
                &mut self.world_positions,
            )
            .with(Player::new(), &mut self.players)
            .with(DamageHistory::default(), &mut self.damage_histories)
            .with(
                HealthUiGraphics {
                    screen_position: Vector2::new(
                        -half_screen_width + HEALTH_UI_SCREEN_PADDING,
                        -half_screen_height + HEALTH_UI_SCREEN_PADDING,
                    ),
                    scale_ratio: 1.0,
                    health: 1.0,
                },
                &mut self.health_ui_graphics,
            )
            .build()
    }

    #[cfg(not(feature = "client"))]
    pub fn create(&mut self) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_z(10.0);

        self.entities
            .build_entity()
            .with(transform, &mut self.transforms)
            .with(PlayerActions::default(), &mut self.player_actions)
            .with(
                WorldPosition::new(Vector2::zero()),
                &mut self.world_positions,
            )
            .with(Player::new(), &mut self.players)
            .with(DamageHistory::default(), &mut self.damage_histories)
            .build()
    }
}

#[derive(SystemData)]
pub struct LandscapeFactory<'s> {
    entities: Entities<'s>,
    #[cfg(feature = "client")]
    asset_handles: ReadExpect<'s, AssetHandles>,
    tags: WriteStorage<'s, Tag<Landscape>>,
    transforms: WriteStorage<'s, Transform>,
    #[cfg(feature = "client")]
    sprite_renders: WriteStorage<'s, SpriteRender>,
}

impl<'s> LandscapeFactory<'s> {
    #[cfg(feature = "client")]
    pub fn create(&mut self) -> Entity {
        let AssetHandles { landscape, .. } = self.asset_handles.clone();

        let mut transform = Transform::default();
        transform.set_translation_z(-1.0);

        self.entities
            .build_entity()
            .with(Tag::<Landscape>::default(), &mut self.tags)
            .with(transform, &mut self.transforms)
            .with(
                SpriteRender {
                    sprite_sheet: landscape,
                    sprite_number: 0,
                },
                &mut self.sprite_renders,
            )
            .build()
    }

    #[cfg(not(feature = "client"))]
    pub fn create(&mut self) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_z(-1.0);

        self.entities
            .build_entity()
            .with(Tag::<Landscape>::default(), &mut self.tags)
            .with(transform, &mut self.transforms)
            .build()
    }
}

#[derive(SystemData)]
pub struct MonsterFactory<'s> {
    entities: Entities<'s>,
    transforms: WriteStorage<'s, Transform>,
    #[cfg(feature = "client")]
    meshes: WriteStorage<'s, Handle<Mesh>>,
    #[cfg(feature = "client")]
    materials: WriteStorage<'s, Handle<Material>>,
    monsters: WriteStorage<'s, Monster>,
    damage_histories: WriteStorage<'s, DamageHistory>,
    world_positions: WriteStorage<'s, WorldPosition>,
}

impl<'s> MonsterFactory<'s> {
    #[cfg(feature = "client")]
    pub fn create(
        &mut self,
        definition: MonsterDefinition,
        position: Vector2,
        destination: Vector2,
        action: MobAction,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 5.0);

        let MonsterDefinition {
            name,
            base_health: health,
            base_speed: _base_speed,
            base_attack_damage: attack_damage,
            graphics: EntityGraphics { mesh, material },
            radius,
            ..
        } = definition;

        self.entities
            .build_entity()
            .with(mesh, &mut self.meshes)
            .with(material, &mut self.materials)
            .with(transform, &mut self.transforms)
            .with(WorldPosition::new(position), &mut self.world_positions)
            .with(
                Monster {
                    health,
                    attack_damage,
                    destination,
                    velocity: Vector2::zero(),
                    action,
                    name,
                    radius,
                },
                &mut self.monsters,
            )
            .with(DamageHistory::default(), &mut self.damage_histories)
            .build()
    }

    #[cfg(not(feature = "client"))]
    pub fn create(
        &mut self,
        definition: MonsterDefinition,
        position: Vector2,
        destination: Vector2,
        action: MobAction,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 5.0);

        let MonsterDefinition {
            name,
            base_health: health,
            base_speed: _base_speed,
            base_attack_damage: attack_damage,
            radius,
            ..
        } = definition;

        self.entities
            .build_entity()
            .with(transform, &mut self.transforms)
            .with(WorldPosition::new(position), &mut self.world_positions)
            .with(
                Monster {
                    health,
                    attack_damage,
                    destination,
                    velocity: Vector2::zero(),
                    action,
                    name,
                    radius,
                },
                &mut self.monsters,
            )
            .with(DamageHistory::default(), &mut self.damage_histories)
            .build()
    }
}

#[derive(SystemData)]
pub struct MissileFactory<'s> {
    entities: Entities<'s>,
    #[cfg(feature = "client")]
    missile_graphics: ReadExpect<'s, MissileGraphics>,
    transforms: WriteStorage<'s, Transform>,
    #[cfg(feature = "client")]
    meshes: WriteStorage<'s, Handle<Mesh>>,
    #[cfg(feature = "client")]
    materials: WriteStorage<'s, Handle<Material>>,
    missiles: WriteStorage<'s, Missile>,
}

impl<'s> MissileFactory<'s> {
    #[cfg(feature = "client")]
    pub fn create(
        &mut self,
        world_positions: &mut WriteStorage<'s, WorldPosition>,
        radius: f32,
        target: MissileTarget,
        velocity: Vector2,
        time_spawned: Duration,
        position: Vector2,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 0.0);

        let EntityGraphics { mesh, material } = self.missile_graphics.0.clone();

        self.entities
            .build_entity()
            .with(mesh.clone(), &mut self.meshes)
            .with(material.clone(), &mut self.materials)
            .with(transform, &mut self.transforms)
            .with(WorldPosition::new(position), world_positions)
            .with(
                Missile::new(radius, target, velocity, time_spawned),
                &mut self.missiles,
            )
            .build()
    }

    #[cfg(not(feature = "client"))]
    pub fn create(
        &mut self,
        world_positions: &mut WriteStorage<'s, WorldPosition>,
        radius: f32,
        target: MissileTarget,
        velocity: Vector2,
        time_spawned: Duration,
        position: Vector2,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz(position.x, position.y, 0.0);

        self.entities
            .build_entity()
            .with(transform, &mut self.transforms)
            .with(WorldPosition::new(position), world_positions)
            .with(
                Missile::new(radius, target, velocity, time_spawned),
                &mut self.missiles,
            )
            .build()
    }
}
