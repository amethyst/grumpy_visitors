#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod application_settings;
mod components;
mod data_resources;
mod factories;
mod models;
mod render_graph;
mod render_groups;
mod states;
mod systems;
mod tags;
mod utils;

pub use crate::utils::math::{Vector2, Vector3, ZeroVector};

use amethyst::{
    animation::AnimationBundle,
    assets::{PrefabLoaderSystem, Processor},
    core::{transform::TransformBundle, HideHierarchySystem},
    input::{InputBundle, StringBindings},
    prelude::{Application, GameDataBuilder},
    renderer::{
        sprite_visibility::SpriteVisibilitySortingSystem, types::DefaultBackend, RenderingSystem,
        SpriteRender, SpriteSheet,
    },
    ui::UiBundle,
    window::WindowBundle,
    LogLevelFilter, Logger,
};

use animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};

use crate::{
    application_settings::ApplicationSettings, components::DamageHistory,
    render_graph::RenderGraph, states::LoadingState, systems::*, utils::animation,
};

fn main() -> amethyst::Result<()> {
    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .start();

    let application_settings = ApplicationSettings::new()?;
    let display_config = application_settings.display().clone();

    let bindings = application_settings.bindings().clone();
    let input_bundle = InputBundle::<StringBindings>::new().with_bindings(bindings);

    let mut builder = Application::build("./", LoadingState::new())?;
    builder.world.add_resource(application_settings);
    builder.world.register::<DamageHistory>();
    let mut damage_history_storage = builder.world.write_storage::<DamageHistory>();
    let game_data = GameDataBuilder::default()
        .with(
            PrefabLoaderSystem::<GameSpriteAnimationPrefab>::default(),
            "",
            &[],
        )
        .with_bundle(WindowBundle::from_config(display_config))?
        .with_bundle(input_bundle)?
        .with(LevelSystem::new(), "level_system", &[])
        .with(SpawnerSystem, "spawner_system", &["level_system"])
        .with(InputSystem::new(), "mouse_system", &["input_system"])
        .with(
            PlayerMovementSystem,
            "player_movement_system",
            &["input_system"],
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
            &["input_system"],
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
        .with(HealthUiSystem, "health_ui_system", &["player_dying_system"])
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
            CameraTranslationSystem,
            "camera_translation_system",
            &["world_position_transform_system"],
        )
        .with(
            AnimationSystem,
            "animation_system",
            &["world_position_transform_system"],
        )
        .with(MenuSystem, "menu_system", &[])
        .with_bundle(TransformBundle::new().with_dep(&["camera_translation_system"]))?
        .with(
            HideHierarchySystem::default(),
            "",
            &["parent_hierarchy_system"],
        )
        .with(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        )
        .with(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &["transform_system"],
        )
        .with_bundle(UiBundle::<DefaultBackend, StringBindings>::new())?
        .with_bundle(
            AnimationBundle::<AnimationId, SpriteRender>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            )
            .with_dep(&["animation_system"]),
        )?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderGraph::default(),
        ));
    drop(damage_history_storage);
    let mut game = builder.build(game_data)?;

    game.run();

    Ok(())
}
