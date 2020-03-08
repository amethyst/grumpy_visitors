use amethyst::{
    ecs::{Entities, Join, ReadStorage},
    prelude::{GameData, SimpleState, StateData},
    renderer::Camera,
};

use gv_core::{
    actions::monster_spawn::SpawnActions,
    ecs::{
        components::{missile::Missile, EntityNetMetadata, Monster, Player},
        resources::{
            net::{CastActionsToExecute, EntityNetMetadataStorage},
            world::{FramedUpdates, WorldStates},
            GameEngineState, GameLevelState,
        },
    },
};

use crate::ecs::systems::{ClientFrameUpdate, FrameUpdate};

pub struct MenuState;

impl SimpleState for MenuState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("MenuState started");
        let world = data.world;
        *world.fetch_mut::<GameEngineState>() = GameEngineState::Menu;

        let mut game_level_state = world.fetch_mut::<GameLevelState>();
        if game_level_state.is_over {
            game_level_state.is_over = false;
            drop(game_level_state);

            world.insert(FramedUpdates::<ClientFrameUpdate>::default());
            world.insert(FramedUpdates::<FrameUpdate>::default());
            world.insert(FramedUpdates::<SpawnActions>::default());
            world.insert(WorldStates::default());
            world.insert(CastActionsToExecute::default());
            world.insert(EntityNetMetadataStorage::new());

            world.exec(
                |(entities, entity_net_metadata, players, monsters, missiles, cameras): (
                    Entities,
                    ReadStorage<EntityNetMetadata>,
                    ReadStorage<Player>,
                    ReadStorage<Monster>,
                    ReadStorage<Missile>,
                    ReadStorage<Camera>,
                )| {
                    for (entity_net_metadata, _) in (&entities, &entity_net_metadata).join() {
                        entities
                            .delete(entity_net_metadata)
                            .expect("Expected to clean up an entity");
                    }
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
}
