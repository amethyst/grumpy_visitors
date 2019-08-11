mod rendering;

pub use ha_core::math::{Vector2, Vector3, ZeroVector};

use amethyst::{
    animation::AnimationBundle,
    assets::PrefabLoaderSystem,
    core::{transform::TransformBundle, HideHierarchySystem},
    input::{InputBundle, StringBindings},
    network::NetworkBundle,
    prelude::{Application, GameDataBuilder},
    renderer::{
        plugins::{RenderFlat2D, RenderFlat3D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle, SpriteRender,
    },
    ui::{RenderUi, UiBundle},
    LogLevelFilter, Logger,
};

use ha_animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};
use ha_client_shared::{ecs::systems::*, settings::Settings};
use ha_game::{build_game_logic_systems, ecs::systems::NetworkingSystem, states::LoadingState};

use crate::rendering::HealthUiPlugin;

fn main() -> amethyst::Result<()> {
    let _cli_matches = clap::App::new("hello_amethyst")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .get_matches();

    let socket_addr = "127.0.0.1:0";

    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .start();

    let settings = Settings::new()?;
    let display_config = settings.display().clone();

    let bindings = settings.bindings().clone();
    let input_bundle = InputBundle::<StringBindings>::new().with_bindings(bindings);

    let mut builder = Application::build("./", LoadingState::default())?;
    builder.world.add_resource(settings);

    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<Vec<u8>>::new(socket_addr.parse()?))?
        .with(NetworkingSystem, "networking_system", &["net_socket"]);

    // Client input systems.
    game_data_builder = game_data_builder
        .with_bundle(input_bundle)?
        .with(
            InputSystem::default(),
            "mouse_system",
            &["networking_system", "input_system"],
        )
        .with(MenuSystem::new(), "menu_system", &[]);

    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, false)?
        .with(
            CameraTranslationSystem,
            "camera_translation_system",
            &["world_position_transform_system"],
        )
        .with_bundle(TransformBundle::new().with_dep(&[
            "world_position_transform_system",
            "camera_translation_system",
        ]))?
        .with(
            PrefabLoaderSystem::<GameSpriteAnimationPrefab>::default(),
            "",
            &[],
        )
        .with(
            HideHierarchySystem::default(),
            "",
            &["parent_hierarchy_system"],
        )
        .with(HealthUiSystem, "health_ui_system", &["player_dying_system"])
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(
            AnimationSystem,
            "animation_system",
            &["world_position_transform_system"],
        )
        .with_bundle(
            AnimationBundle::<AnimationId, SpriteRender>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            )
            .with_dep(&["animation_system"]),
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config(display_config))
                .with_plugin(RenderFlat3D::default())
                .with_plugin(RenderFlat2D::default())
                .with_plugin(HealthUiPlugin::default())
                .with_plugin(RenderUi::default()),
        )?;

    let mut game = builder.build(game_data_builder)?;

    game.run();

    Ok(())
}
