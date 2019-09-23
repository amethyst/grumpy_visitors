#![allow(clippy::type_complexity)]

mod ecs;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    network::NetworkBundle,
    prelude::{Application, GameDataBuilder},
    LogLevelFilter, LoggerConfig, StdoutLog,
};
use log::LevelFilter;

use std::{env, io, path::PathBuf, str::FromStr, time::Duration};

use ha_core::{
    ecs::resources::world::{FramedUpdates, PlayerActionUpdates, ServerWorldUpdates},
    net::EncodedMessage,
};
use ha_game::{
    build_game_logic_systems, ecs::systems::NetConnectionManagerSystem, states::LoadingState,
};

use crate::ecs::systems::*;

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
                .default_value("127.0.0.1:3455")
                .takes_value(true),
        )
        .get_matches();

    let socket_addr = cli_matches
        .value_of("addr")
        .expect("Expected a default value");

    Logger::from_config(Default::default())
        .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
        .level_for("ha_game::ecs::systems", LogLevelFilter::Debug)
        .level_for(
            "ha_game::ecs::systems::net_connection_manager",
            LogLevelFilter::Trace,
        )
        .start();

    let mut builder = Application::build("./", LoadingState::default())?;
    builder
        .world
        .add_resource(FramedUpdates::<PlayerActionUpdates>::default());
    builder.world.add_resource(ServerWorldUpdates::default());
    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<EncodedMessage>::new(socket_addr.parse()?))?
        .with(
            NetConnectionManagerSystem::new(),
            "net_connection_manager_system",
            &["net_socket"],
        )
        .with(
            ServerNetworkSystem::new(),
            "game_network_system",
            &["net_socket"],
        );
    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, true)?
        .with(
            GameUpdatesBroadcastingSystem::default(),
            "game_updates_broadcasting_system",
            &["action_system"],
        )
        .with_bundle(TransformBundle::new().with_dep(&["world_position_transform_system"]))?;

    let mut game = builder
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60,
        )
        .build(game_data_builder)?;
    game.run();

    Ok(())
}

// TODO: watch https://github.com/amethyst/amethyst/issues/1889 to get rid of this copy-pasta.
pub struct Logger {
    dispatch: fern::Dispatch,
}

impl Logger {
    fn new() -> Self {
        let dispatch = fern::Dispatch::new().format(|out, message, record| {
            out.finish(format_args!(
                "[{level}][SERVER][{target}] {message}",
                level = record.level(),
                target = record.target(),
                message = message,
            ))
        });
        Self { dispatch }
    }

    /// Create a new Logger from [`LoggerConfig`]
    pub fn from_config(mut config: LoggerConfig) -> Self {
        if config.allow_env_override {
            env_var_override(&mut config);
        }

        let mut logger = Self::new();
        logger.dispatch = logger.dispatch.level(config.level_filter);

        match config.stdout {
            StdoutLog::Plain => logger.dispatch = logger.dispatch.chain(io::stdout()),
            StdoutLog::Colored => {
                logger.dispatch = logger
                    .dispatch
                    .chain(colored_stdout(fern::colors::ColoredLevelConfig::new()))
            }
            StdoutLog::Off => {}
        }

        if let Some(log_gfx_device_level) = config.log_gfx_device_level {
            logger.dispatch = logger
                .dispatch
                .level_for("gfx_device_gl", log_gfx_device_level);
        }

        if let Some(path) = config.log_file {
            if let Ok(log_file) = fern::log_file(path) {
                logger.dispatch = logger.dispatch.chain(log_file)
            } else {
                eprintln!("Unable to access the log file, as such it will not be used")
            }
        }

        logger
    }

    /// Set individual log levels for modules.
    pub fn level_for<T: Into<std::borrow::Cow<'static, str>>>(
        mut self,
        module: T,
        level: LevelFilter,
    ) -> Self {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    /// Starts [`Logger`] by consuming it.
    pub fn start(self) {
        self.dispatch.apply().unwrap_or_else(|_| {
            log::debug!("Global logger already set, default Amethyst logger will not be used")
        });
    }
}

fn env_var_override(config: &mut LoggerConfig) {
    if let Ok(var) = env::var("AMETHYST_LOG_STDOUT") {
        match var.to_lowercase().as_ref() {
            "off" | "no" | "0" => config.stdout = StdoutLog::Off,
            "plain" | "yes" | "1" => config.stdout = StdoutLog::Plain,
            "colored" | "2" => config.stdout = StdoutLog::Colored,
            _ => {}
        }
    }
    if let Ok(var) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
        if let Ok(lf) = LevelFilter::from_str(&var) {
            config.level_filter = lf;
        }
    }
    if let Ok(path) = env::var("AMETHYST_LOG_FILE_PATH") {
        config.log_file = Some(PathBuf::from(path));
    }
}

fn colored_stdout(color_config: fern::colors::ColoredLevelConfig) -> fern::Dispatch {
    fern::Dispatch::new()
        .chain(io::stdout())
        .format(move |out, message, record| {
            let color = color_config.get_color(&record.level());
            out.finish(format_args!(
                "{color}{message}{color_reset}",
                color = format!("\x1B[{}m", color.to_fg_str()),
                message = message,
                color_reset = "\x1B[0m",
            ))
        })
}
