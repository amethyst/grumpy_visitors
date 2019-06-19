use amethyst::{
    assets::{AssetStorage, Handle, Loader, PrefabLoader, ProgressCounter, RonFormat},
    ecs::World,
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{FontAsset, TtfFormat},
    utils::tag::Tag,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{
    animation,
    components::{Missile, Player, WorldPosition},
    data_resources::{GameScene, MissileGraphics, MonsterDefinitions},
    factories::{create_menu_screen, create_player},
    models::{AssetsHandles, GameState, SpawnActions},
    states::PlayingState,
    tags::*,
    utils::camera::initialise_camera,
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
        log::info!("LoadingState started");
        let world = data.world;

        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();
        world.register::<Tag<UiBackground>>();
        world.register::<Tag<Landscape>>();

        MissileGraphics::register(world);
        MonsterDefinitions::register(world);
        world.add_resource(SpawnActions(Vec::new()));
        world.add_resource(GameScene::default());
        world.add_resource(GameState::Loading);

        let ui_font_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            let font_storage = world.read_resource::<AssetStorage<FontAsset>>();

            let font_handle = loader.load(
                "resources/PT_Sans-Web-Regular.ttf",
                TtfFormat,
                &mut self.progress_counter,
                &font_storage,
            );

            font_handle
        };

        let landscape_handle = load_sprite_sheet(
            world,
            "resources/levels/desert.png",
            "resources/levels/desert.ron",
        );

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

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}
