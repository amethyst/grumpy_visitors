#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod components;
mod data_resources;
mod factories;
mod missiles_system;
mod models;
mod players_movement_system;
mod systems;

use amethyst::{
    core::transform::{Transform, TransformBundle},
    input::{is_close_requested, InputBundle},
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat, Pipeline, PosTex, Projection, RenderBundle, Stage,
        WindowMessages,
    },
    utils::application_root_dir,
};
use winit::{ElementState, VirtualKeyCode};

use crate::{
    components::*,
    data_resources::{MissileGraphics, MonsterDefinitions},
    factories::create_player,
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions},
    players_movement_system::PlayersMovementSystem,
    systems::{InputSystem, MonsterActionSystem, MonsterMovementSystem, SpawnerSystem},
};

struct HelloAmethyst {
    pub fullscreen: bool,
}

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

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;

        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                return Trans::Quit;
            }

            if let winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if input.virtual_keycode == Some(VirtualKeyCode::F11)
                    && input.state == ElementState::Released
                {
                    let mut window_messages = world.write_resource::<WindowMessages>();
                    let is_fullscreen = self.fullscreen;
                    self.fullscreen = !self.fullscreen;
                    window_messages.send_command(move |window| {
                        let monitor_id = if is_fullscreen {
                            None
                        } else {
                            window.get_available_monitors().next()
                        };
                        window.set_fullscreen(monitor_id);
                    });
                }
            }

            Trans::None
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config_path = application_root_dir()
        .unwrap()
        .join("resources/display_config.ron");
    let display_config = DisplayConfig::load(&display_config_path);
    let fullscreen = display_config.fullscreen;

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
        .with(InputSystem::new(), "mouse_system", &["input_system"])
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
            &["monster_action_system"],
        )
        .with(
            MissilesSystem,
            "missiles_system",
            &["mouse_system", "players_movement_system"],
        );
    let mut game = Application::new("./", HelloAmethyst { fullscreen }, game_data)?;

    game.run();

    Ok(())
}

fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 1024.0, 0.0, 768.0,
        )))
        .with(transform)
        .build();
}
