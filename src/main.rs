#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod components;
mod data_resources;
mod factories;
mod missiles_system;
mod models;
mod mouse_system;
mod players_movement_system;
mod systems;

use amethyst::{
    core::transform::{Transform, TransformBundle},
    input::InputBundle,
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat, Pipeline, PosTex, Projection, RenderBundle, Stage,
    },
    utils::application_root_dir,
};

use crate::{
    components::*,
    data_resources::{MissileGraphics, MonsterDefinitions},
    factories::create_player,
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions},
    mouse_system::MouseSystem,
    players_movement_system::PlayersMovementSystem,
    systems::SpawnerSystem,
};
use crate::systems::{MonsterMovementSystem, MonsterActionSystem};

struct HelloAmethyst;

type Vector2 = amethyst::core::math::Vector2<f32>;
type Vector3 = amethyst::core::math::Vector3<f32>;

impl SimpleState for HelloAmethyst {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();

        MissileGraphics::register(world);
        MonsterDefinitions::register(world);
        world.add_resource(SpawnActions(vec![SpawnAction {
            monsters: Count {
                entity: "Ghoul".to_owned(),
                num: 1,
            },
        }]));

        initialise_camera(world);
        create_player(world);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = application_root_dir()
        .unwrap()
        .join("resources/display_config.ron");
    let display_config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let bindings_config_path = application_root_dir()
        .unwrap()
        .join("resources/bindings_config.ron");
    let input_bundle =
        InputBundle::<String, String>::new().with_bindings_from_file(bindings_config_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::new(pipe, Some(display_config)))?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(SpawnerSystem, "spawner_system", &[])
        .with(MouseSystem::new(), "mouse_system", &["input_system"])
        .with(
            PlayersMovementSystem,
            "players_movement_system",
            &["input_system"],
        )
        .with(
            MonsterActionSystem,
            "monster_action_system",
            &["players_movement_system"],
        )
        .with(
            MonsterMovementSystem,
            "monster_movement_system",
            &["monster_action_system"]
        )
        .with(
            MissilesSystem,
            "missiles_system",
            &["mouse_system", "players_movement_system"],
        );
    let mut game = Application::new("./", HelloAmethyst, game_data)?;

    game.run();

    Ok(())
}

pub const ARENA_WIDTH: f32 = 1024.0;
pub const ARENA_HEIGHT: f32 = 768.0;

fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            0.0,
            ARENA_HEIGHT,
        )))
        .with(transform)
        .build();
}
