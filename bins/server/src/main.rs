use amethyst::{
    core::transform::TransformBundle,
    network::NetworkBundle,
    prelude::{Application, GameDataBuilder},
    LogLevelFilter, Logger,
};

use ha_game::{build_game_logic_systems, ecs::systems::NetworkingSystem, states::LoadingState};

fn main() -> amethyst::Result<()> {
    let cli_matches = clap::App::new("hello_amethyst")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDR")
                .help("Specifies the address for UdpSocket")
                .default_value("0.0.0.0:3455")
                .takes_value(true),
        )
        .get_matches();

    let socket_addr = cli_matches
        .value_of("addr")
        .expect("Expected a default value");

    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .start();

    let mut builder = Application::build("./", LoadingState::default())?;
    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<Vec<u8>>::new(socket_addr.parse()?))?
        .with(NetworkingSystem, "networking_system", &["net_socket"]);
    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, true)?
        .with_bundle(TransformBundle::new().with_dep(&["world_position_transform_system"]))?;

    let mut game = builder.build(game_data_builder)?;
    game.run();

    Ok(())
}
