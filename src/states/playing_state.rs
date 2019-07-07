use amethyst::prelude::{GameData, SimpleState, SimpleTrans, StateData, StateEvent, Trans};

use crate::{
    animation,
    factories::create_player,
    factories::{create_debug_scene_border, create_landscape},
    models::{
        common::{AssetsHandles, GameState},
        monster_spawn::{Count, SpawnAction, SpawnActions, SpawnType},
    },
    utils::{self, camera::initialise_camera},
};

#[derive(Default)]
pub struct PlayingState;

impl SimpleState for PlayingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("PlayingState started");
        let world = data.world;
        *world.write_resource::<GameState>() = GameState::Playing;

        let AssetsHandles { hero_prefab, .. } = world.read_resource::<AssetsHandles>().clone();

        let player = create_player(world, hero_prefab);
        initialise_camera(world, player);

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

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        animation::start_hero_animations(data.world);
        Trans::None
    }
}
