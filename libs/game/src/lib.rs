#![allow(clippy::type_complexity, clippy::too_many_arguments)]
pub mod ecs;
pub mod states;
pub mod utils;

use amethyst::{
    error::Error,
    prelude::{GameDataBuilder, World},
};

use ha_core::ecs::{components::damage_history::DamageHistory, resources::MultiplayerRoomPlayers};

use crate::ecs::{
    resources::IncomingMessages,
    systems::{missile::*, monster::*, player::*, *},
};

pub fn build_game_logic_systems<'a, 'b>(
    game_data_builder: GameDataBuilder<'a, 'b>,
    world: &mut World,
    is_server: bool,
) -> Result<GameDataBuilder<'a, 'b>, Error> {
    world.add_resource(IncomingMessages(Vec::new()));
    world.add_resource(MultiplayerRoomPlayers::new());

    world.register::<DamageHistory>();
    let mut damage_history_storage = world.write_storage::<DamageHistory>();

    let game_data_builder = game_data_builder
        .with(
            NetConnectionManagerSystem,
            "net_connection_manager_system",
            &["net_socket"],
        )
        .with(LevelSystem::default(), "level_system", &[])
        .with(MonsterSpawnerSystem, "spawner_system", &["level_system"])
        .with(
            PlayerMovementSystem,
            "player_movement_system",
            &dependencies_with_optional(&[], !is_server, &["input_system"]),
        )
        .with(
            MonsterActionSystem,
            "monster_action_system",
            &["player_movement_system"],
        )
        .with(
            MonsterMovementSystem,
            "monster_movement_system",
            &["monster_action_system"],
        )
        .with(
            MissileSpawnerSystem,
            "missile_spawner_system",
            &dependencies_with_optional(&[], !is_server, &["input_system"]),
        )
        .with(
            MissileSystem,
            "missile_system",
            &["missile_spawner_system", "player_movement_system"],
        )
        .with(
            MonsterDyingSystem::new(damage_history_storage.register_reader()),
            "monster_dying_system",
            &["missile_system"],
        )
        .with(
            PlayerDyingSystem::new(damage_history_storage.register_reader()),
            "player_dying_system",
            &["missile_system", "monster_action_system"],
        )
        .with(
            WorldPositionTransformSystem,
            "world_position_transform_system",
            &[
                "missile_system",
                "player_movement_system",
                "monster_movement_system",
            ],
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
