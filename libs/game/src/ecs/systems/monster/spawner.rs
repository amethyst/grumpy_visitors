use amethyst::{
    ecs::{Entity, ReadExpect, System, World, WriteExpect, WriteStorage},
    shred::{ResourceId, SystemData},
};

use ha_core::{
    actions::{
        mob::MobAction,
        monster_spawn::{SpawnAction, SpawnActions, SpawnType},
        Action,
    },
    ecs::{
        components::EntityNetMetadata,
        resources::{net::EntityNetMetadataStorage, world::FramedUpdates, GameLevelState},
        system_data::time::GameTimeService,
    },
    math::{Vector2, ZeroVector},
    net::NetIdentifier,
};

use crate::{
    ecs::{
        factories::MonsterFactory,
        resources::{MonsterDefinition, MonsterDefinitions},
        system_data::GameStateHelper,
        systems::{AggregatedOutcomingUpdates, FrameUpdate, OutcomingNetUpdates},
    },
    utils::world::{outcoming_net_updates_mut, spawning_side},
};

#[derive(SystemData)]
pub struct MonsterSpawnerSystemData<'s> {
    pub game_time_service: GameTimeService<'s>,
    pub game_state_helper: GameStateHelper<'s>,
    pub monster_definitions: ReadExpect<'s, MonsterDefinitions>,
    pub game_level_state: ReadExpect<'s, GameLevelState>,
    pub entity_net_metadata: WriteStorage<'s, EntityNetMetadata>,
    pub entity_net_metadata_storage: WriteExpect<'s, EntityNetMetadataStorage>,
    pub monster_factory: MonsterFactory<'s>,
}

pub struct MonsterSpawnerSystem;

impl<'s> System<'s> for MonsterSpawnerSystem {
    type SystemData = (
        WriteExpect<'s, AggregatedOutcomingUpdates>,
        ReadExpect<'s, FramedUpdates<FrameUpdate>>,
        WriteExpect<'s, FramedUpdates<SpawnActions>>,
        MonsterSpawnerSystemData<'s>,
    );

    fn run(
        &mut self,
        (mut aggregated_outcoming_updates, framed_updates, mut spawn_actions, mut system_data): Self::SystemData,
    ) {
        if !system_data.game_state_helper.is_running() {
            return;
        }

        // A hackish way to sync oldest_updated_frame on server side.
        spawn_actions.oldest_updated_frame = spawn_actions
            .oldest_updated_frame
            .min(framed_updates.oldest_updated_frame);

        for spawn_actions in spawn_actions.iter_from_oldest_update() {
            if spawn_actions.frame_number > system_data.game_time_service.game_frame_number() {
                break;
            }

            system_data.spawn_monsters(
                spawn_actions,
                outcoming_net_updates_mut(
                    &mut *aggregated_outcoming_updates,
                    spawn_actions.frame_number,
                    system_data.game_time_service.game_frame_number(),
                ),
            );
        }
        spawn_actions.oldest_updated_frame = system_data.game_time_service.game_frame_number();
    }
}

impl<'s> MonsterSpawnerSystemData<'s> {
    pub fn spawn_monsters(
        &mut self,
        spawn_actions: &SpawnActions,
        outcoming_net_updates: &mut OutcomingNetUpdates,
    ) {
        if !self.game_state_helper.is_running() {
            return;
        }

        let frame_number = spawn_actions.frame_number;
        if self.game_state_helper.is_multiplayer() && self.game_state_helper.is_authoritative() {
            Self::add_action_updates(outcoming_net_updates, spawn_actions.spawn_actions.clone());
        }
        let spawn_actions = self.get_spawn_actions(&spawn_actions);

        for spawn_action in spawn_actions {
            let ghoul = self
                .monster_definitions
                .0
                .get("Ghoul")
                .expect("Failed to get Ghoul monster definition")
                .clone();

            match spawn_action.spawn_type {
                SpawnType::Single {
                    entity_net_id,
                    position,
                } => {
                    self.spawn_monster(
                        frame_number,
                        position,
                        Action {
                            frame_number,
                            action: MobAction::Idle,
                        },
                        &ghoul,
                        entity_net_id,
                    );
                }
                SpawnType::Borderline {
                    count,
                    mut entity_net_id_range,
                    side,
                } => {
                    let (side_start, side_end, destination) =
                        spawning_side(side, &self.game_level_state);
                    let spawn_distance = (side_end - side_start) / f32::from(count);

                    let mut position = side_start;
                    for _ in 0..count {
                        let action = Action {
                            frame_number,
                            action: MobAction::Move(position + destination),
                        };
                        self.spawn_monster(
                            frame_number,
                            position,
                            action,
                            &ghoul,
                            entity_net_id_range.as_mut().map(|entity_net_id_range| {
                                entity_net_id_range
                                    .next()
                                    .expect("Expected a reserved EntityIdentifier")
                            }),
                        );
                        position += spawn_distance;
                    }
                }
            }
        }
    }

    fn get_spawn_actions(&mut self, spawn_actions: &SpawnActions) -> Vec<SpawnAction> {
        if self.game_state_helper.is_multiplayer() {
            spawn_actions
                .spawn_actions
                .iter()
                .cloned()
                .filter(|action| {
                    // Filter out already spawned entities.
                    let entity_net_id = match &action.spawn_type {
                        SpawnType::Single { entity_net_id, .. } => *entity_net_id,
                        SpawnType::Borderline {
                            entity_net_id_range,
                            ..
                        } => entity_net_id_range
                            .as_ref()
                            .map(|range| range.end.saturating_sub(1)),
                    };
                    entity_net_id
                        .and_then(|entity_net_id| {
                            self.entity_net_metadata_storage.get_entity(entity_net_id)
                        })
                        .is_none()
                })
                .collect()
        } else {
            spawn_actions.spawn_actions.clone()
        }
    }

    #[cfg(feature = "client")]
    fn add_action_updates(
        _outcoming_net_update: &mut OutcomingNetUpdates,
        _spawn_actions: Vec<SpawnAction>,
    ) {
    }

    #[cfg(not(feature = "client"))]
    fn add_action_updates(
        outcoming_net_update: &mut OutcomingNetUpdates,
        spawn_actions: Vec<SpawnAction>,
    ) {
        outcoming_net_update.spawn_actions = spawn_actions;
    }

    fn spawn_monster(
        &mut self,
        frame_number: u64,
        position: Vector2,
        action: Action<MobAction<Entity>>,
        monster_definition: &MonsterDefinition,
        net_id: Option<NetIdentifier>,
    ) {
        log::trace!("Spawning a monster with net id {:?}", net_id);
        let destination = if let MobAction::Move(destination) = action.action {
            destination
        } else {
            Vector2::zero()
        };
        let monster_entity = self.monster_factory.create(
            frame_number,
            monster_definition.clone(),
            position,
            destination,
            action,
        );

        if let Some(net_id) = net_id {
            self.entity_net_metadata
                .insert(
                    monster_entity,
                    EntityNetMetadata {
                        id: net_id,
                        spawned_frame_number: frame_number,
                    },
                )
                .expect("Expected to insert EntityNetMetadata");

            self.entity_net_metadata_storage
                .set_net_id(monster_entity, net_id);
        }
    }
}
