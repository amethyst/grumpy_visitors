#[cfg(feature = "client")]
use amethyst::{
    assets::ProgressCounter,
    assets::{AssetStorage, Handle, Loader, PrefabLoader, RonFormat},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{FontAsset, TtfFormat, UiCreator},
};
use amethyst::{
    ecs::{prelude::WorldExt, World},
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    utils::tag::Tag,
};

#[cfg(feature = "client")]
use gv_animation_prefabs::GameSpriteAnimationPrefab;
#[cfg(feature = "client")]
use gv_client_shared::ecs::{
    components::HealthUiGraphics,
    resources::{AssetHandles, DummyAssetHandles, HealthUiMesh, MissileGraphics},
};
use gv_core::ecs::{
    components::{missile::Missile, Player, WorldPosition},
    resources::{GameEngineState, GameLevelState, GameTime, NewGameEngineState},
    tags::*,
};

use crate::ecs::resources::MonsterDefinitions;

#[cfg(feature = "client")]
#[derive(Default)]
pub struct LoadingState {
    progress_counter: ProgressCounter,
    atlas_progress_counter: ProgressCounter,
    atlas_is_loaded: bool,
    rest_is_loaded: bool,
}

#[cfg(not(feature = "client"))]
#[derive(Default)]
pub struct LoadingState;

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("LoadingState started");
        let world = data.world;

        self.register_client_dependencies(world);
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
        if self.register_client_dependencies(data.world) {
            *data.world.write_resource::<NewGameEngineState>() =
                NewGameEngineState(GameEngineState::Menu);
        }
        Trans::None
    }
}

impl LoadingState {
    #[cfg(feature = "client")]
    fn register_client_dependencies(&mut self, world: &mut World) -> bool {
        match (
            self.atlas_is_loaded,
            self.atlas_progress_counter.is_complete(),
            self.rest_is_loaded,
            self.progress_counter.is_complete(),
        ) {
            (false, _, _, _) => {
                self.atlas_is_loaded = true;
                let dummy_prefab = world.exec(
                    |prefab_loader: PrefabLoader<'_, GameSpriteAnimationPrefab>| {
                        prefab_loader.load(
                            "resources/prefabs/dummy.ron",
                            RonFormat,
                            &mut self.atlas_progress_counter,
                        )
                    },
                );
                world.insert(DummyAssetHandles { dummy_prefab });
                false
            }
            (true, true, false, _) => {
                self.rest_is_loaded = true;
                world.register::<HealthUiGraphics>();
                MissileGraphics::register(world);
                HealthUiMesh::register(world);

                let ui_font_handle = {
                    let loader = world.read_resource::<Loader>();
                    let font_storage = world.read_resource::<AssetStorage<FontAsset>>();

                    loader.load(
                        "resources/PT_Sans-Web-Regular.ttf",
                        TtfFormat,
                        &mut self.progress_counter,
                        &font_storage,
                    )
                };

                let landscape_handle = load_sprite_sheet(
                    world,
                    "resources/assets/desert_level.png",
                    "resources/levels/desert.ron",
                    &mut self.progress_counter,
                );

                let (mage_prefab, beetle_prefab) = world.exec(
                    |prefab_loader: PrefabLoader<'_, GameSpriteAnimationPrefab>| {
                        let mage_prefab = prefab_loader.load(
                            "resources/prefabs/mage.ron",
                            RonFormat,
                            &mut self.progress_counter,
                        );
                        let beetle_prefab = prefab_loader.load(
                            "resources/prefabs/beetle.ron",
                            RonFormat,
                            &mut self.progress_counter,
                        );
                        (mage_prefab, beetle_prefab)
                    },
                );

                let _ui_handle =
                    world.exec(|mut creator: UiCreator| creator.create("resources/ui/hud.ron", ()));
                let _ui_handles = world.exec(|mut creator: UiCreator| {
                    (
                        creator.create("resources/ui/main_menu.ron", ()),
                        creator.create("resources/ui/lobby_menu.ron", ()),
                        creator.create("resources/ui/multiplayer_menu.ron", ()),
                        creator.create("resources/ui/restart_menu.ron", ()),
                    )
                });

                world.insert(AssetHandles {
                    mage_prefab,
                    beetle_prefab,
                    landscape: landscape_handle,
                    ui_font: ui_font_handle,
                });

                false
            }
            (true, true, true, is_complete) => is_complete,
            _ => false,
        }
    }

    #[cfg(not(feature = "client"))]
    fn register_client_dependencies(&mut self, _world: &mut World) -> bool {
        true
    }
}

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
