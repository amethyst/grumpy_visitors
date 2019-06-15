use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, ProgressCounter, RonFormat},
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    renderer::{ImageFormat, Texture},
    ui::{FontAsset, TtfFormat},
    utils::tag::Tag,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{
    animation,
    components::{Missile, Player, WorldPosition},
    data_resources::{GameScene, MissileGraphics, MonsterDefinitions},
    factories::{create_menu_screen, create_player},
    utils::camera::initialise_camera,
    models::{AssetsHandles, GameState, SpawnActions},
    states::PlayingState,
    tags::UiBackground,
};

pub struct LoadingState {
    pub progress_counter: ProgressCounter,
}

impl LoadingState {
    pub fn new() -> Self {
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
                ImageFormat::default(),
                &mut self.progress_counter,
                &texture_storage,
            );
            let font_handle = loader.load(
                "resources/PT_Sans-Web-Regular.ttf",
                TtfFormat,
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
            Trans::Switch(Box::new(PlayingState))
        } else {
            Trans::None
        }
    }
}
