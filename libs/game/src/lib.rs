#![feature(maybe_uninit_extra)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
pub mod ecs;
pub mod states;
pub mod utils;

use amethyst::{
    error::Error,
    prelude::{GameDataBuilder, World, WorldExt},
};

use ha_core::ecs::{
    components::damage_history::DamageHistory,
    resources::{
        net::{ActionUpdateIdProvider, EntityNetMetadataStorage, MultiplayerGameState},
        world::{FramedUpdates, PlayerActionUpdates, WorldStates},
    },
};

use crate::ecs::{
    resources::ConnectionEvents,
    systems::{missile::*, monster::*, player::*, *},
};

pub fn build_game_logic_systems<'a, 'b>(
    game_data_builder: GameDataBuilder<'a, 'b>,
    world: &mut World,
    is_server: bool,
) -> Result<GameDataBuilder<'a, 'b>, Error> {
    world.insert(FramedUpdates::<PlayerActionUpdates>::default());
    world.insert(WorldStates::default());
    world.insert(ConnectionEvents(Vec::new()));
    world.insert(MultiplayerGameState::new());
    world.insert(EntityNetMetadataStorage::new());
    world.insert(ActionUpdateIdProvider::default());

    world.register::<DamageHistory>();
    let mut damage_history_storage = world.write_storage::<DamageHistory>();

    let game_data_builder = game_data_builder
        .with(PauseSystem, "pause_system", &["game_network_system"])
        .with(LevelSystem::default(), "level_system", &["pause_system"])
        .with(MonsterSpawnerSystem, "spawner_system", &["level_system"])
        .with(
            MissileSpawnerSystem,
            "missile_spawner_system",
            &dependencies_with_optional(&["level_system"], !is_server, &["input_system"]),
        )
        .with(
            ActionSystem,
            "action_system",
            &dependencies_with_optional(
                &["missile_spawner_system", "spawner_system"],
                !is_server,
                &["input_system"],
            ),
        )
        .with(MissileSystem, "missile_system", &["action_system"])
        .with(
            MonsterDyingSystem::new(damage_history_storage.register_reader()),
            "monster_dying_system",
            &["missile_system"],
        )
        .with(
            PlayerDyingSystem::new(damage_history_storage.register_reader()),
            "player_dying_system",
            &["action_system"],
        )
        .with(
            WorldPositionTransformSystem,
            "world_position_transform_system",
            &["action_system"],
        )
        .with(
            StateSwitcherSystem,
            "state_switcher_system",
            &optional_dependencies(&["menu_system"], !is_server),
        );
    Ok(game_data_builder)
}

fn optional_dependencies(dependencies: &[&'static str], condition: bool) -> Vec<&'static str> {
    if condition {
        dependencies.to_vec()
    } else {
        Vec::new()
    }
}

fn dependencies_with_optional(
    mandatory: &[&'static str],
    condition: bool,
    optional: &[&'static str],
) -> Vec<&'static str> {
    let mut dependencies = mandatory.to_vec();
    dependencies.append(&mut optional_dependencies(optional, condition));
    dependencies
}
