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
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationControlSet, AnimationSet,
        EndControl,
    },
    assets::{PrefabLoader, PrefabLoaderSystem, ProgressCounter, RonFormat},
    core::{
        math::Orthographic3,
        transform::{Parent, Transform, TransformBundle},
    },
    ecs::{Entities, Entity, Join, ReadStorage, WriteStorage},
    input::{is_close_requested, InputBundle},
    prelude::*,
    renderer::{
        Camera, DrawFlat, DrawFlat2D, Pipeline, PosTex, Projection, RenderBundle, ScreenDimensions,
        SpriteRender, Stage, WindowMessages,
    },
};
use winit::{ElementState, VirtualKeyCode};

use animation_prefabs::{AnimationId, GameSpritePrefab};

use crate::{
    application_settings::ApplicationSettings,
    components::*,
    data_resources::*,
    factories::{create_debug_scene_border, create_player},
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions, SpawnType},
    players_movement_system::PlayersMovementSystem,
    systems::*,
};

#[derive(Default)]
struct HelloAmethyst {
    pub progress_counter: Option<ProgressCounter>,
}

type Vector2 = amethyst::core::math::Vector2<f32>;
type Vector3 = amethyst::core::math::Vector3<f32>;

impl SimpleState for HelloAmethyst {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.progress_counter = Some(Default::default());
        // Starts asset loading
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, GameSpritePrefab>| {
            loader.load(
                "resources/animation_metadata.ron",
                RonFormat,
                (),
                self.progress_counter.as_mut().unwrap(),
            )
        });

        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();

        MissileGraphics::register(world);
        MonsterDefinitions::register(world);
        world.add_resource(SpawnActions(vec![
            SpawnAction {
                monsters: Count {
                    entity: "Ghoul".to_owned(),
                    num: 1,
                },
                spawn_type: SpawnType::Borderline,
            },
            SpawnAction {
                monsters: Count {
                    entity: "Ghoul".to_owned(),
                    num: 5,
                },
                spawn_type: SpawnType::Random,
            },
        ]));
        world.add_resource(GameScene::default());

        let player = create_player(world, prefab_handle);
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
                    let mut transforms = world.write_storage::<Transform>();
                    let (mut camera, camera_transform) =
                        (&mut cameras, &mut transforms).join().next().unwrap();
                    let (screen_width, screen_height) = (size.width as f32, size.height as f32);

                    camera.proj =
                        Orthographic3::new(0.0, screen_width, 0.0, screen_height, 0.1, 2000.0)
                            .to_homogeneous();
                    camera_transform.set_translation(Vector3::new(
                        -screen_width / 2.0,
                        -screen_height / 2.0,
                        1.0,
                    ));
                }

                _ => {}
            };
        }
        Trans::None
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Checks if we are still loading data
        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let StateData { world, .. } = data;
                // Execute a pass similar to a system
                world.exec(
                    |(entities, animation_sets, mut control_sets): (
                        Entities,
                        ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                        WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
                    )| {
                        // For each entity that has AnimationSet
                        for (entity, animation_set) in (&entities, &animation_sets).join() {
                            // Creates a new AnimationControlSet for the entity
                            let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                            // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                            control_set.add_animation(
                                AnimationId::Walk,
                                &animation_set.get(&AnimationId::Walk).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                        }
                    },
                );
                // All data loaded
                self.progress_counter = None;
            }
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
            .clear_target([0.10196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new())
            .with_pass(DrawFlat2D::new()),
    );

    let bindings = application_settings.bindings().clone();
    let input_bundle = InputBundle::<String, String>::new().with_bindings(bindings);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<GameSpritePrefab>::default(), "", &[])
        .with_bundle(RenderBundle::new(pipe, Some(display_config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(AnimationBundle::<AnimationId, SpriteRender>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
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
        )
        .with(
            CameraTranslationSystem,
            "camera_translation_system",
            &["players_movement_system"],
        );
    let mut builder = Application::build("./", HelloAmethyst::default())?;
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
