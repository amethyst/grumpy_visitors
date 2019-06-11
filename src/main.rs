#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod application_settings;
mod components;
mod data_resources;
mod factories;
mod missiles_system;
mod models;
mod players_movement_system;
mod systems;
mod tags;
mod utils;

use amethyst::{
    animation::AnimationBundle,
    assets::{
        AssetStorage, Handle, Loader, Prefab, PrefabLoader, PrefabLoaderSystem, ProgressCounter,
        RonFormat,
    },
    core::transform::{Parent, Transform, TransformBundle},
    ecs::Entity,
    input::InputBundle,
    prelude::*,
    renderer::{
        Camera, DrawFlat, DrawFlat2D, HideHierarchySystem, Pipeline, PngFormat, PosTex, Projection,
        RenderBundle, ScreenDimensions, SpriteRender, Stage, Texture, TextureHandle,
        TextureMetadata,
    },
    ui::{DrawUi, FontAsset, FontHandle, TtfFormat, UiBundle},
    utils::tag::Tag,
};

use animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};

use crate::factories::create_menu_screen;
use crate::models::GameState;
use crate::{
    application_settings::ApplicationSettings,
    components::*,
    data_resources::*,
    factories::{create_debug_scene_border, create_landscape, create_player},
    missiles_system::MissilesSystem,
    models::{Count, SpawnAction, SpawnActions, SpawnType},
    players_movement_system::PlayersMovementSystem,
    systems::*,
    tags::*,
    utils::animation,
};

struct LoadingState {
    pub progress_counter: ProgressCounter,
}

impl LoadingState {
    fn new() -> Self {
        Self {
            progress_counter: Default::default(),
        }
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();
        world.register::<Tag<UiBackground>>();

        MissileGraphics::register(world);
        MonsterDefinitions::register(world);
        world.add_resource(SpawnActions(Vec::new()));
        world.add_resource(GameScene::default());
        world.add_resource(GameState::Loading);

        let (landscape_handle, ui_font_handle) = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            let font_storage = world.read_resource::<AssetStorage<FontAsset>>();

            let landscape_handle = loader.load(
                "resources/levels/desert.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                &mut self.progress_counter,
                &texture_storage,
            );
            let font_handle = loader.load(
                "resources/PT_Sans-Web-Regular.ttf",
                TtfFormat,
                (),
                &mut self.progress_counter,
                &font_storage,
            );

            (landscape_handle, font_handle)
        };

        let hero_prefab_handle = world.exec(
            |prefab_loader: PrefabLoader<'_, GameSpriteAnimationPrefab>| {
                prefab_loader.load(
                    "resources/animation_metadata.ron",
                    RonFormat,
                    (),
                    &mut self.progress_counter,
                )
            },
        );

        let player = create_player(world, hero_prefab_handle.clone());
        initialise_camera(world, player);
        create_menu_screen(world, ui_font_handle.clone());

        world.add_resource(AssetsHandles {
            hero_prefab: hero_prefab_handle,
            landscape: landscape_handle,
            ui_font: ui_font_handle,
        });
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { ref mut world, .. } = data;
        if self.progress_counter.is_complete() {
            animation::start_hero_animations(world);
            Trans::Switch(Box::new(HelloAmethyst))
        } else {
            Trans::None
        }
    }
}

#[derive(Clone)]
struct AssetsHandles {
    hero_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    landscape: TextureHandle,
    ui_font: FontHandle,
}

#[derive(Default)]
struct HelloAmethyst;

type Vector2 = amethyst::core::math::Vector2<f32>;
type Vector3 = amethyst::core::math::Vector3<f32>;

impl SimpleState for HelloAmethyst {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        *world.write_resource::<GameState>() = GameState::Playing;

        {
            let mut spawn_actions = world.write_resource::<SpawnActions>();
            spawn_actions.0.append(&mut vec![
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
            ]);
        }

        let AssetsHandles { landscape, .. } = world.read_resource::<AssetsHandles>().clone();

        create_landscape(world, landscape);
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

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let application_settings = ApplicationSettings::new()?;

    let display_config = application_settings.display().clone();

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new())
            .with_pass(DrawFlat2D::new())
            .with_pass(DrawUi::new()),
    );

    let bindings = application_settings.bindings().clone();
    let input_bundle = InputBundle::<String, String>::new().with_bindings(bindings);

    let game_data = GameDataBuilder::default()
        .with(
            PrefabLoaderSystem::<GameSpriteAnimationPrefab>::default(),
            "",
            &[],
        )
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
            AnimationSystem,
            "animation_system",
            &["players_movement_system", "monster_movement_system"],
        )
        .with(MenuSystem, "menu_system", &[])
        .with_bundle(
            TransformBundle::new()
                .with_dep(&["players_movement_system", "monster_movement_system"]),
        )?
        .with(
            HideHierarchySystem::default(),
            "",
            &["parent_hierarchy_system"],
        )
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&["transform_system"]),
        )?
        .with_bundle(
            AnimationBundle::<AnimationId, SpriteRender>::new(
                "animation_control_system",
                "sampler_interpolation_system",
            )
            .with_dep(&["animation_system"]),
        )?
        .with(
            CameraTranslationSystem,
            "camera_translation_system",
            &["players_movement_system"],
        );
    let mut builder = Application::build("./", LoadingState::new())?;
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
