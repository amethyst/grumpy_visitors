use amethyst::{
    assets::ProgressCounter,
    ecs::{prelude::WorldExt, World},
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    utils::tag::Tag,
};
#[cfg(feature = "client")]
use amethyst::{
    assets::{AssetStorage, Handle, Loader, PrefabLoader, RonFormat},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{FontAsset, TtfFormat, UiCreator},
};

#[cfg(feature = "client")]
use gv_animation_prefabs::GameSpriteAnimationPrefab;
#[cfg(feature = "client")]
use gv_client_shared::ecs::{
    components::HealthUiGraphics,
    resources::{AssetHandles, HealthUiMesh, MissileGraphics},
};
use gv_core::ecs::{
    components::{missile::Missile, Player, WorldPosition},
    resources::{GameEngineState, GameLevelState, GameTime, NewGameEngineState},
    tags::*,
};

use crate::ecs::resources::MonsterDefinitions;

#[derive(Default)]
pub struct LoadingState {
    pub progress_counter: ProgressCounter,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("LoadingState started");
        let world = data.world;

        register_client_dependencies(world, &mut self.progress_counter);
        MonsterDefinitions::register(world);
        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();
        world.register::<Tag<Landscape>>();
        world.insert(GameLevelState::default());
        world.insert(GameTime::default());
        world.insert(GameEngineState::Loading);
        world.insert(NewGameEngineState(GameEngineState::Loading));
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            *data.world.write_resource::<NewGameEngineState>() =
                NewGameEngineState(GameEngineState::Menu);
        }
        Trans::None
    }
}

#[cfg(feature = "client")]
fn register_client_dependencies(world: &mut World, progress_counter: &mut ProgressCounter) {
    world.register::<HealthUiGraphics>();

    MissileGraphics::register(world);
    HealthUiMesh::register(world);

    let ui_font_handle = {
        let loader = world.read_resource::<Loader>();
        let font_storage = world.read_resource::<AssetStorage<FontAsset>>();

        loader.load(
            "resources/PT_Sans-Web-Regular.ttf",
            TtfFormat,
            &mut *progress_counter,
            &font_storage,
        )
    };

    let landscape_handle = load_sprite_sheet(
        world,
        "resources/levels/desert.png",
        "resources/levels/desert.ron",
        &mut *progress_counter,
    );

    let (mage_prefab, beetle_prefab) = world.exec(
        |prefab_loader: PrefabLoader<'_, GameSpriteAnimationPrefab>| {
            let mage_prefab =
                prefab_loader.load("resources/mage.ron", RonFormat, &mut *progress_counter);
            let beetle_prefab =
                prefab_loader.load("resources/beetle.ron", RonFormat, &mut *progress_counter);
            (mage_prefab, beetle_prefab)
        },
    );

    let _ui_handle =
        world.exec(|mut creator: UiCreator| creator.create("resources/ui/hud.ron", ()));
    let _ui_handle =
        world.exec(|mut creator: UiCreator| creator.create("resources/ui/main_menu.ron", ()));

    world.insert(AssetHandles {
        mage_prefab,
        beetle_prefab,
        landscape: landscape_handle,
        ui_font: ui_font_handle,
    });
}

#[cfg(not(feature = "client"))]
fn register_client_dependencies(_world: &mut World, _progress_counter: &mut ProgressCounter) {}

#[cfg(feature = "client")]
fn load_sprite_sheet(
    world: &mut World,
    png_path: &str,
    ron_path: &str,
    progress: &mut ProgressCounter,
) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            ImageFormat::default(),
            &mut *progress,
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        progress,
        &sprite_sheet_store,
    )
}
