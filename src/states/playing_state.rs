use amethyst::prelude::{GameData, SimpleState, SimpleTrans, StateData, StateEvent, Trans};

use crate::{
    factories::{create_debug_scene_border, create_landscape},
    models::{AssetsHandles, Count, GameState, SpawnAction, SpawnActions, SpawnType},
    utils,
};

#[derive(Default)]
pub struct PlayingState;

impl SimpleState for PlayingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("PlayingState started");
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
