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
    pub spawn_actions: WriteExpect<'s, SpawnActions>,
    pub entity_net_metadata: WriteStorage<'s, EntityNetMetadata>,
    pub entity_net_metadata_storage: WriteExpect<'s, EntityNetMetadataStorage>,
    pub monster_factory: MonsterFactory<'s>,
}

pub struct MonsterSpawnerSystem;

impl<'s> System<'s> for MonsterSpawnerSystem {
    type SystemData = (
        WriteExpect<'s, FramedUpdates<FrameUpdate>>,
        WriteExpect<'s, AggregatedOutcomingUpdates>,
        MonsterSpawnerSystemData<'s>,
    );

    fn run(
        &mut self,
        (mut framed_updates, mut aggregated_outcoming_updates, mut system_data): Self::SystemData,
    ) {
        if !system_data.game_state_helper.is_running() {
            return;
        }

        framed_updates.reserve_updates(system_data.game_time_service.game_frame_number());

        for frame_updates in framed_updates.iter_from_oldest_update() {
            system_data.spawn_monsters(
                frame_updates,
                outcoming_net_updates_mut(
                    &mut *aggregated_outcoming_updates,
                    frame_updates.frame_number,
                    system_data.game_time_service.game_frame_number(),
                ),
            );
        }
    }
}

impl<'s> MonsterSpawnerSystemData<'s> {
    pub fn spawn_monsters(
        &mut self,
        frame_updates: &FrameUpdate,
        outcoming_net_updates: &mut OutcomingNetUpdates,
    ) {
        if !self.game_state_helper.is_running() {
            return;
        }

        let frame_number = frame_updates.frame_number;
        let spawn_actions = self.get_spawn_actions(&frame_updates);
        if self.game_state_helper.is_multiplayer()
            && self.game_state_helper.is_authoritative()
            && self.game_time_service.game_frame_number() == frame_number
        {
            Self::add_action_updates(outcoming_net_updates, spawn_actions.clone());
        }

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

    #[cfg(feature = "client")]
    fn get_spawn_actions(&mut self, frame_updates: &FrameUpdate) -> Vec<SpawnAction> {
        if self.game_state_helper.is_multiplayer() {
            frame_updates
                .spawn_actions
                .iter()
                .cloned()
                .filter(|action| {
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
            self.spawn_actions.0.drain(..).collect()
        }
    }

    #[cfg(not(feature = "client"))]
    fn get_spawn_actions(&mut self, frame_updates: &FrameUpdate) -> Vec<SpawnAction> {
        if self.game_time_service.game_frame_number() == frame_updates.frame_number {
            self.spawn_actions.0.drain(..).collect()
        } else {
            Vec::new()
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
        let monster_entity =
            self.monster_factory
                .create(monster_definition.clone(), position, destination, action);

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
