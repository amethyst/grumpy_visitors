#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod ecs;
mod rendering;
mod utils;

use amethyst::{
    animation::AnimationBundle,
    assets::PrefabLoaderSystem,
    core::{
        frame_limiter::FrameRateLimitStrategy, transform::TransformBundle, HideHierarchySystem,
    },
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

use std::time::Duration;

use ha_animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};
use ha_client_shared::{ecs::resources::MultiplayerRoomState, settings::Settings};
use ha_core::{
    ecs::resources::world::{ClientWorldUpdates, FramedUpdates, ServerWorldUpdate},
    net::EncodedMessage,
};
use ha_game::{
    build_game_logic_systems, ecs::systems::NetConnectionManagerSystem, states::LoadingState,
};

use crate::{
    ecs::{
        resources::{LastAcknowledgedUpdate, ServerCommand},
        systems::*,
    },
    rendering::HealthUiPlugin,
};

fn main() -> amethyst::Result<()> {
    let _cli_matches = clap::App::new("hello_amethyst")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .get_matches();

    let socket_addr = "127.0.0.1:0";

    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .level_for("ha_game::ecs::systems", LogLevelFilter::Debug)
        .level_for(
            "ha_game::ecs::systems::net_connection_manager",
            LogLevelFilter::Info,
        )
        .level_for("ha_client", LogLevelFilter::Debug)
        .start();

    let settings = Settings::new()?;
    let display_config = settings.display().clone();

    let bindings = settings.bindings().clone();
    let input_bundle = InputBundle::<StringBindings>::new().with_bindings(bindings);

    let mut builder = Application::build("./", LoadingState::default())?;
    builder.world.add_resource(settings);
    builder.world.add_resource(ServerCommand::new());
    builder.world.add_resource(MultiplayerRoomState::new());
    builder.world.add_resource(ClientWorldUpdates::default());
    builder.world.add_resource(LastAcknowledgedUpdate(0));
    builder
        .world
        .add_resource(FramedUpdates::<ServerWorldUpdate>::default());

    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<EncodedMessage>::new(socket_addr.parse()?))?
        .with(
            NetConnectionManagerSystem::default(),
            "net_connection_manager_system",
            &["net_socket"],
        )
        .with(
            ClientNetworkSystem,
            "game_network_system",
            &["net_connection_manager_system"],
        )
        .with_bundle(input_bundle)?
        .with(InputSystem::default(), "mouse_system", &["input_system"])
        .with(MenuSystem::new(), "menu_system", &[])
        .with(LocalServerSystem, "local_server_system", &["menu_system"]);

    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, false)?
        .with(
            GameUpdatesBroadcastingSystem::default(),
            "game_updates_broadcasting_system",
            &["action_system"],
        )
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

    let mut game = builder
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60,
        )
        .build(game_data_builder)?;

    game.run();

    Ok(())
}
