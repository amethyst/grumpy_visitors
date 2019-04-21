#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod application_settings;
mod components;
mod data_resources;
mod factories;
mod missiles_system;
mod models;
mod players_movement_system;
mod systems;

use amethyst::{
    core::{
        math::Orthographic3,
        transform::{Parent, Transform, TransformBundle},
    },
    ecs::{Entity, Join},
    input::{is_close_requested, InputBundle},
    prelude::*,
    renderer::{
        Camera, DrawFlat, Pipeline, PosTex, Projection, RenderBundle, ScreenDimensions, Stage,
        WindowMessages,
    },
};
use winit::{ElementState, VirtualKeyCode};

use crate::{
    application_settings::ApplicationSettings,
    components::*,
    data_resources::*,
    factories::{create_debug_scene_border, create_player},
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions},
    players_movement_system::PlayersMovementSystem,
    systems::*,
};

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
        world.add_resource(GameScene::default());

        let player = create_player(world);
        initialise_camera(world, player);
        create_debug_scene_border(world);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;
        let mut application_settings = world.write_resource::<ApplicationSettings>();
        let display = application_settings.display();

        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                return Trans::Quit;
            }

            match event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::KeyboardInput { input, .. },
                    ..
                } if input.state == ElementState::Released => match input.virtual_keycode {
                    Some(VirtualKeyCode::F11) => {
                        let mut window_messages = world.write_resource::<WindowMessages>();
                        let is_fullscreen = display.fullscreen;
                        application_settings
                            .save_fullscreen(!is_fullscreen)
                            .expect("Failed to save settings");

                        window_messages.send_command(move |window| {
                            let monitor_id = if is_fullscreen {
                                None
                            } else {
                                window.get_available_monitors().next()
                            };
                            window.set_fullscreen(monitor_id);
                        });
                    }
                    Some(VirtualKeyCode::F10) => {
                        let screen_dimensions = world.read_resource::<ScreenDimensions>();
                        println!(
                            "{}:{}",
                            screen_dimensions.width(),
                            screen_dimensions.height()
                        );
                    }
                    _ => {}
                },

                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Resized(size),
                    ..
                } => {
                    let mut cameras = world.write_storage::<Camera>();
                    let camera: &mut Camera = (&mut cameras).join().next().unwrap();

                    camera.proj = Orthographic3::new(
                        0.0,
                        size.width as f32,
                        0.0,
                        size.height as f32,
                        0.1,
                        2000.0,
                    )
                    .to_homogeneous();
                }

                _ => {}
            };
        }
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let application_settings = ApplicationSettings::new()?;

    let display_config = application_settings.display().clone();

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let bindings = application_settings.bindings().clone();
    let input_bundle = InputBundle::<String, String>::new().with_bindings(bindings);

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
    let mut builder = Application::build("./", HelloAmethyst)?;
    builder.world.add_resource(application_settings);
    let mut game = builder.build(game_data)?;

    game.run();

    Ok(())
}

fn initialise_camera(world: &mut World, player: Entity) {
    let transform = {
        let screen_dimensions = world.read_resource::<ScreenDimensions>();
        let mut transform = Transform::default();
        transform.set_translation(Vector3::new(
            -screen_dimensions.width() / 2.0,
            -screen_dimensions.height() / 2.0,
            1.0,
        ));
        transform
    };

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 1024.0, 0.0, 768.0,
        )))
        .with(transform)
        .with(Parent::new(player))
        .build();
}
