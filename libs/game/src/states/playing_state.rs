#[cfg(feature = "client")]
use amethyst::prelude::{SimpleTrans, StateEvent, Trans};
use amethyst::{
    ecs::{Entity, World},
    prelude::{GameData, SimpleState, StateData},
    shred::SystemData,
};

#[cfg(feature = "client")]
use ha_client_shared::{
    ecs::factories::CameraFactory,
    utils::{self, animation},
};
use ha_core::{
    actions::monster_spawn::{Count, SpawnAction, SpawnActions, SpawnType},
    ecs::{
        resources::{GameEngineState, GameLevelState},
        system_data::time::GameTimeService,
    },
};

use crate::ecs::factories::{LandscapeFactory, PlayerFactory};

#[derive(Default)]
pub struct PlayingState;

impl SimpleState for PlayingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("PlayingState started");
        let world = data.world;
        *world.write_resource::<GameEngineState>() = GameEngineState::Playing;

        world.add_resource(SpawnActions(Vec::new()));
        world.add_resource(GameLevelState::default());

        GameTimeService::fetch(&world.res).set_level_started_at();

        let player = world.exec(|mut player_factory: PlayerFactory| player_factory.create());
        initialize_camera(world, player);

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

        world.exec(|mut landscape_factory: LandscapeFactory| landscape_factory.create());
    }

    #[cfg(feature = "client")]
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;
        utils::handle_window_event(&world, &event);
        Trans::None
    }

    #[cfg(feature = "client")]
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        animation::start_hero_animations(data.world);
        Trans::None
    }
}

#[cfg(feature = "client")]
fn initialize_camera(world: &mut World, player: Entity) {
    world.exec(move |mut camera_factory: CameraFactory| camera_factory.create(player));
}

#[cfg(not(feature = "client"))]
fn initialize_camera(_world: &mut World, _player: Entity) {}
