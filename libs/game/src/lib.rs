#![feature(drain_filter)]
#![feature(clamp)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod ecs;
pub mod states;
pub mod utils;

use amethyst::{
    error::Error,
    prelude::{GameDataBuilder, World},
};

use gv_core::{
    actions::monster_spawn::SpawnActions,
    ecs::resources::{
        net::{
            ActionUpdateIdProvider, CastActionsToExecute, EntityNetMetadataStorage,
            MultiplayerGameState,
        },
        world::{FramedUpdates, PlayerActionUpdates, WorldStates},
    },
};

use crate::ecs::{
    resources::ConnectionEvents,
    systems::{missile::MissileDyingSystem, monster::*, *},
};

pub static PLAYER_COLORS: [[f32; 3]; 5] = [
    [0.64, 0.12, 0.11],
    [0.04, 0.45, 0.69],
    [0.0, 0.49, 0.26],
    [0.40, 0.3, 0.55],
    [0.57, 0.57, 0.57],
];

pub fn build_game_logic_systems<'a, 'b>(
    game_data_builder: GameDataBuilder<'a, 'b>,
    world: &mut World,
    is_server: bool,
) -> Result<GameDataBuilder<'a, 'b>, Error> {
    world.insert(ConnectionEvents(Vec::new()));
    world.insert(MultiplayerGameState::new());
    world.insert(ActionUpdateIdProvider::default());

    // The resources which we need to remember to reset on starting a game.
    world.insert(FramedUpdates::<PlayerActionUpdates>::default());
    world.insert(FramedUpdates::<SpawnActions>::default());
    world.insert(WorldStates::default());
    world.insert(CastActionsToExecute::default());
    world.insert(EntityNetMetadataStorage::new());

    let game_data_builder = game_data_builder
        .with(PauseSystem, "pause_system", &["game_network_system"])
        .with(LevelSystem::default(), "level_system", &["pause_system"])
        .with(MonsterSpawnerSystem, "spawner_system", &["level_system"])
        .with(
            ActionSystem,
            "action_system",
            &dependencies_with_optional(&["spawner_system"], !is_server, &["input_system"]),
        )
        .with(
            MonsterDyingSystem,
            "monster_dying_system",
            &["action_system"],
        )
        .with(
            MissileDyingSystem,
            "missile_dying_system",
            &["action_system"],
        )
        .with(
            StateSwitcherSystem,
            "state_switcher_system",
            &dependencies_with_optional(
                &["monster_dying_system", "missile_dying_system"],
                !is_server,
                &["menu_system"],
            ),
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
