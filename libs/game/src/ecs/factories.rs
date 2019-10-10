#![allow(clippy::type_repetition_in_bounds)]

#[cfg(feature = "client")]
use amethyst::{
    assets::Handle,
    ecs::ReadExpect,
    renderer::{Material, Mesh, SpriteRender},
};
use amethyst::{
    core::Transform,
    ecs::{prelude::World, Entities, Entity, WriteStorage},
    shred::{ResourceId, SystemData},
    utils::tag::Tag,
};

#[cfg(feature = "client")]
use ha_client_shared::ecs::resources::{AssetHandles, EntityGraphics};
use ha_core::{
    actions::{mob::MobAction, Action},
    ecs::{
        components::{damage_history::DamageHistory, *},
        tags::*,
    },
    math::{Vector2, ZeroVector},
};

use crate::ecs::resources::MonsterDefinition;

#[derive(SystemData)]
pub struct PlayerFactory<'s> {
    entities: Entities<'s>,
    transforms: WriteStorage<'s, Transform>,
    player_actions: WriteStorage<'s, PlayerActions>,
    world_positions: WriteStorage<'s, WorldPosition>,
    net_world_positions: WriteStorage<'s, NetWorldPosition>,
    players: WriteStorage<'s, Player>,
    player_last_casted_spells: WriteStorage<'s, PlayerLastCastedSpells>,
    damage_histories: WriteStorage<'s, DamageHistory>,
}

impl<'s> PlayerFactory<'s> {
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
            .with(
                NetWorldPosition::new(Vector2::zero()),
                &mut self.net_world_positions,
            )
            .with(Player::new(), &mut self.players)
            .with(
                PlayerLastCastedSpells::default(),
                &mut self.player_last_casted_spells,
            )
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
    pub entities: Entities<'s>,
    pub transforms: WriteStorage<'s, Transform>,
    #[cfg(feature = "client")]
    pub meshes: WriteStorage<'s, Handle<Mesh>>,
    #[cfg(feature = "client")]
    pub materials: WriteStorage<'s, Handle<Material>>,
    pub monsters: WriteStorage<'s, Monster>,
    pub damage_histories: WriteStorage<'s, DamageHistory>,
    pub world_positions: WriteStorage<'s, WorldPosition>,
}

impl<'s> MonsterFactory<'s> {
    #[cfg(feature = "client")]
    pub fn create(
        &mut self,
        definition: MonsterDefinition,
        position: Vector2,
        destination: Vector2,
        action: Action<MobAction<Entity>>,
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
        action: Action<MobAction<Entity>>,
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
