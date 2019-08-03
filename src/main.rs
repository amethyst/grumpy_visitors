mod rendering;

pub use ha_core::math::{Vector2, Vector3, ZeroVector};

use amethyst::{
    animation::AnimationBundle,
    assets::PrefabLoaderSystem,
    core::{transform::TransformBundle, HideHierarchySystem},
    error::Error,
    input::{InputBundle, StringBindings},
    network::NetworkBundle,
    prelude::{Application, GameDataBuilder, World},
    renderer::{
        plugins::{RenderFlat2D, RenderFlat3D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle, SpriteRender,
    },
    ui::{RenderUi, UiBundle},
    LogLevelFilter, Logger,
};

use ha_animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};
use ha_game::{
    application_settings::ApplicationSettings,
    ecs::{
        components::damage_history::DamageHistory,
        systems::{missile::*, monster::*, player::*, ui::*, *},
    },
    states::LoadingState,
};

use crate::rendering::HealthUiPlugin;

fn main() -> amethyst::Result<()> {
    let cli_matches = clap::App::new("hello_amethyst")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .arg(
            clap::Arg::with_name("server")
                .short("s")
                .long("server")
                .value_name("SERVER")
                .help("Start the game in headless server mode")
                .takes_value(false))
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDR")
                .help("Specifies the address for UdpSocket (defaults: (client) 127.0.0.1:0, (server) 0.0.0.0:3455)")
                .takes_value(true))
        .get_matches();

    let is_server = cli_matches.is_present("server");
    let socket_addr = cli_matches.value_of("addr").unwrap_or_else(|| {
        if is_server {
            "0.0.0.0:3455"
        } else {
            "127.0.0.1:0"
        }
    });

    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .start();

    let application_settings = ApplicationSettings::new()?;
    let display_config = application_settings.display().clone();

    let bindings = application_settings.bindings().clone();
    let input_bundle = InputBundle::<StringBindings>::new().with_bindings(bindings);

    let mut builder = Application::build("./", LoadingState::default())?;
    builder.world.add_resource(application_settings);

    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<Vec<u8>>::new(socket_addr.parse()?))?
        .with(NetworkingSystem, "networking_system", &["net_socket"]);

    // Client input systems.
    if !is_server {
        game_data_builder = game_data_builder.with_bundle(input_bundle)?.with(
            InputSystem::default(),
            "mouse_system",
            &dependencies_with_optional(&["networking_system"], !is_server, &["input_system"]),
        );
    }

    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, is_server)?;

    if !is_server {
        game_data_builder = game_data_builder.with(
            CameraTranslationSystem,
            "camera_translation_system",
            &["world_position_transform_system"],
        )
    }

    game_data_builder = game_data_builder.with_bundle(TransformBundle::new().with_dep(
        &dependencies_with_optional(
            &["world_position_transform_system"],
            !is_server,
            &["camera_translation_system"],
        ),
    ))?;

    if !is_server {
        game_data_builder = game_data_builder
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
            .with(MenuSystem::new(), "menu_system", &[])
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
    }
    let mut game = builder.build(game_data_builder)?;

    game.run();

    Ok(())
}

fn build_game_logic_systems<'a, 'b>(
    game_data_builder: GameDataBuilder<'a, 'b>,
    world: &mut World,
    is_server: bool,
) -> Result<GameDataBuilder<'a, 'b>, Error> {
    world.register::<DamageHistory>();
    let mut damage_history_storage = world.write_storage::<DamageHistory>();
    let game_data_builder = game_data_builder
        .with(LevelSystem::default(), "level_system", &[])
        .with(MonsterSpawnerSystem, "spawner_system", &["level_system"])
        .with(
            PlayerMovementSystem,
            "player_movement_system",
            &dependencies_with_optional(&["networking_system"], !is_server, &["input_system"]),
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
            &dependencies_with_optional(&["networking_system"], !is_server, &["input_system"]),
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
