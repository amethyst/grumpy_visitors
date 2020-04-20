#![allow(clippy::type_complexity)]

mod ecs;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    network::simulation::laminar::{LaminarConfig, LaminarNetworkBundle, LaminarSocket},
    prelude::{Application, GameDataBuilder, SystemDesc},
    Logger, LoggerConfig,
};

use gv_core::ecs::resources::world::{
    DummyFramedUpdate, FramedUpdates, ReceivedClientActionUpdates, ServerWorldUpdates,
};
use gv_game::{
    build_game_logic_systems,
    ecs::systems::{NetConnectionManagerDesc, WorldPositionTransformSystem},
    states::LoadingState,
};

use crate::ecs::{
    resources::{HostClientAddress, LastBroadcastedFrame},
    systems::*,
};

fn main() -> amethyst::Result<()> {
    let cli_matches = clap::App::new("grumpy_visitors")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDR")
                .help("Specifies the address for UdpSocket")
                .default_value("127.0.0.1:3455")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("host-client-addr")
                .short("c")
                .long("client-addr")
                .value_name("CLIENT_ADDR")
                .help("Specifies the address of the client hosting the game")
                .takes_value(true),
        )
        .get_matches();

    let socket_addr = cli_matches
        .value_of("addr")
        .expect("Expected a default value if not passed via CLI");
    let client_addr = cli_matches.value_of("host-client-addr");
    let client_addr = if let Some(client_addr) = client_addr {
        HostClientAddress(Some(client_addr.parse()?))
    } else {
        HostClientAddress(None)
    };

    let logging_config: LoggerConfig = ::std::fs::read_to_string("server_logging_config.toml")
        .map_err(|err| {
            log::warn!(
                "Failed to read server_logging_config.toml, using the defaults: {:?}",
                err
            )
        })
        .and_then(|config_contents| {
            toml::from_str(&config_contents).map_err(|err| {
                log::warn!(
                    "Failed to read server_logging_config.toml, using the defaults: {:?}",
                    err
                )
            })
        })
        .unwrap_or_default();
    Logger::from_config_formatter(logging_config, |out, message, record| {
        out.finish(format_args!(
            "[{level}][SERVER][{target}] {message}",
            level = record.level(),
            target = record.target(),
            message = message,
        ))
    })
    .start();

    let mut builder = Application::build("./", LoadingState::default())?;
    builder
        .world
        .insert(FramedUpdates::<DummyFramedUpdate>::default());
    builder
        .world
        .insert(FramedUpdates::<ReceivedClientActionUpdates>::default());
    builder.world.insert(client_addr);
    builder.world.insert(ServerWorldUpdates::default());
    builder.world.insert(LastBroadcastedFrame(0));

    let laminar_config = LaminarConfig {
        receive_buffer_max_size: 14_500,
        ..LaminarConfig::default()
    };

    let socket = LaminarSocket::bind_with_config(socket_addr, laminar_config)?;

    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with(
            NetConnectionManagerDesc::default().build(&mut builder.world),
            "net_connection_manager_system",
            &[],
        )
        .with(ServerNetworkSystem::new(), "game_network_system", &[]);
    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, true)?
        .with(
            WorldPositionTransformSystem,
            "world_position_transform_system",
            &["action_system"],
        )
        .with(
            GameUpdatesBroadcastingSystem::default(),
            "game_updates_broadcasting_system",
            &["action_system"],
        )
        .with_bundle(TransformBundle::new().with_dep(&["world_position_transform_system"]))?;

    let mut game = builder
        .with_frame_limit(FrameRateLimitStrategy::Yield, 60)
        .build(game_data_builder)?;
    game.run();
    Ok(())
}
