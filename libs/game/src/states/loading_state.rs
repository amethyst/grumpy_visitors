use amethyst::{
    assets::{AssetStorage, Handle, Loader, PrefabLoader, ProgressCounter, RonFormat},
    ecs::World,
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{FontAsset, TtfFormat, UiCreator},
    utils::tag::Tag,
};

use ha_animation_prefabs::GameSpriteAnimationPrefab;

use crate::{
    actions::monster_spawn::SpawnActions,
    ecs::{
        components::{missile::Missile, HealthUiGraphics, Player, WorldPosition},
        resources::{
            graphics::{HealthUiMesh, MissileGraphics},
            AssetHandles, GameEngineState, GameLevelState, GameTime, MonsterDefinitions,
        },
        tags::*,
    },
    states::MenuState,
};

#[derive(Default)]
pub struct LoadingState {
    pub progress_counter: ProgressCounter,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("LoadingState started");
        let world = data.world;

        world.register::<WorldPosition>();
        world.register::<Missile>();
        world.register::<Player>();
        world.register::<Tag<Landscape>>();
        world.register::<HealthUiGraphics>();

        MissileGraphics::register(world);
        MonsterDefinitions::register(world);
        HealthUiMesh::register(world);
        world.add_resource(SpawnActions(Vec::new()));
        world.add_resource(GameLevelState::default());
        world.add_resource(GameTime::default());
        world.add_resource(GameEngineState::Loading);

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
            "resources/levels/desert.png",
            "resources/levels/desert.ron",
            &mut self.progress_counter,
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

        let _ui_handle =
            world.exec(|mut creator: UiCreator| creator.create("resources/ui/hud.ron", ()));
        let _ui_handle =
            world.exec(|mut creator: UiCreator| creator.create("resources/ui/main_menu.ron", ()));

        world.add_resource(AssetHandles {
            hero_prefab: hero_prefab_handle,
            landscape: landscape_handle,
            ui_font: ui_font_handle,
        });
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            Trans::Switch(Box::new(MenuState))
        } else {
            Trans::None
        }
    }
}

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
