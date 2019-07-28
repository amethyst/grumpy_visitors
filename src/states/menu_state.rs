use amethyst::{
    ecs::{Entities, Join, ReadStorage},
    prelude::{GameData, SimpleState, SimpleTrans, StateData, Trans},
    renderer::Camera,
};

use crate::{
    components::{Missile, Monster, Player},
    data_resources::{GameEngineState, GameLevelState},
    states::PlayingState,
};

pub struct MenuState;

impl SimpleState for MenuState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("MenuState started");
        let world = data.world;
        *world.write_resource::<GameEngineState>() = GameEngineState::Menu;

        let mut game_level_state = world.write_resource::<GameLevelState>();
        if game_level_state.is_over {
            game_level_state.is_over = false;
            drop(game_level_state);
            world.exec(
                |(entities, players, monsters, missiles, cameras): (
                    Entities,
                    ReadStorage<Player>,
                    ReadStorage<Monster>,
                    ReadStorage<Missile>,
                    ReadStorage<Camera>,
                )| {
                    for (player_entity, _) in (&entities, &players).join() {
                        entities
                            .delete(player_entity)
                            .expect("Expected to clean up an entity");
                    }
                    for (monster_entity, _) in (&entities, &monsters).join() {
                        entities
                            .delete(monster_entity)
                            .expect("Expected to clean up an entity");
                    }
                    for (missile_entity, _) in (&entities, &missiles).join() {
                        entities
                            .delete(missile_entity)
                            .expect("Expected to clean up an entity");
                    }
                    for (camera_entity, _) in (&entities, &cameras).join() {
                        entities
                            .delete(camera_entity)
                            .expect("Expected to clean up an entity");
                    }
                },
            );
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = &data.world;
        if let GameEngineState::Playing = *world.read_resource::<GameEngineState>() {
            Trans::Switch(Box::new(PlayingState))
        } else {
            Trans::None
        }
    }
}
