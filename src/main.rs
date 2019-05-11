#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod application_settings;
mod components;
mod data_resources;
mod factories;
mod missiles_system;
mod models;
mod players_movement_system;
mod systems;
mod utils;

use amethyst::{
    animation::AnimationBundle,
    assets::{PrefabLoaderSystem, ProgressCounter},
    core::transform::{Parent, Transform, TransformBundle},
    ecs::Entity,
    input::InputBundle,
    prelude::*,
    renderer::{
        Camera, DrawFlat, DrawFlat2D, Pipeline, PosTex, Projection, RenderBundle, ScreenDimensions,
        SpriteRender, Stage,
    },
};

use animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};

use crate::utils::animation::update_loading_prefab;
use crate::{
    application_settings::ApplicationSettings,
    components::*,
    data_resources::*,
    factories::{create_debug_scene_border, create_player},
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions, SpawnType},
    players_movement_system::PlayersMovementSystem,
    systems::*,
    utils::animation,
};

#[derive(Default)]
struct HelloAmethyst {
    pub progress_counter: Option<ProgressCounter>,
}

type Vector2 = amethyst::core::math::Vector2<f32>;
type Vector3 = amethyst::core::math::Vector3<f32>;

impl SimpleState for HelloAmethyst {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;

        self.progress_counter = Some(Default::default());
        let hero_prefab_handle = animation::load_prefab(&mut world, &mut self.progress_counter);

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

        let player = create_player(world, hero_prefab_handle);
        initialise_camera(world, player);
        create_debug_scene_border(world);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;
        utils::handle_window_event(&world, &event);
        Trans::None
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { ref mut world, .. } = data;
        update_loading_prefab(world, &mut self.progress_counter);
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
        .with(
            PrefabLoaderSystem::<GameSpriteAnimationPrefab>::default(),
            "",
            &[],
        )
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&[]),
        )?
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
