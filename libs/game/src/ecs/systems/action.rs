#[cfg(feature = "client")]
use amethyst::{
    animation::{AnimationControlSet, AnimationSet},
    assets::Handle,
    core::{Named, ParentHierarchy},
    renderer::{sprite::SpriteRender, Material, Mesh},
};
use amethyst::{
    core::{HiddenPropagate, Transform},
    ecs::{
        Entities, Entity, Join, ReadExpect, ReadStorage, System, World, WriteExpect, WriteStorage,
    },
    shred::{ResourceId, SystemData},
};
use gv_core::profile_scope;

#[cfg(not(feature = "client"))]
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "client")]
use gv_animation_prefabs::AnimationId;
use gv_core::{
    actions::{
        mob::MobAction,
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
        ClientActionUpdate, IdentifiableAction,
    },
    ecs::{
        components::{
            damage_history::{DamageHistory, DamageHistoryEntries},
            missile::Missile,
            ClientPlayerActions, Dead, EntityNetMetadata, Monster, NetWorldPosition, Player,
            PlayerActions, PlayerLastCastedSpells, WorldPosition,
        },
        resources::{
            net::{
                ActionUpdateIdProvider, CastActionsToExecute, EntityNetMetadataStorage,
                MultiplayerGameState,
            },
            world::{FramedUpdates, SavedWorldState, WorldStates},
            GameLevelState,
        },
        system_data::time::GameTimeService,
    },
    net::{NetUpdate, INTERPOLATION_FRAME_DELAY},
};

use crate::{
    ecs::{
        resources::MonsterDefinitions,
        system_data::GameStateHelper,
        systems::{
            missile::{MissileFactory, MissilePhysicsSubsystem, MissileSpawnerSubsystem},
            monster::{ApplyMonsterActionNetArgs, MonsterActionSubsystem},
            player::{
                ApplyCastActionNetArgs, ApplyLookActionNetArgs, ApplyWalkActionNetArgs,
                PlayerActionSubsystem,
            },
            world_state_subsystem::WorldStateSubsystem,
            AggregatedOutcomingUpdates, AnimationsResourceBundle, ClientFrameUpdate,
            DamageSubsystem, FrameUpdate, GraphicsResourceBundle,
        },
    },
    utils::{entities::is_dead, world::outcoming_net_updates_mut},
};

#[derive(SystemData)]
pub struct ActionSystemData<'s> {
    entities: Entities<'s>,
    game_time_service: GameTimeService<'s>,
    game_state_helper: GameStateHelper<'s>,
    graphics_system_data: GraphicsSystemData<'s>,
    animations_system_data: AnimationsSystemData<'s>,
    game_level_state: ReadExpect<'s, GameLevelState>,
    multiplayer_game_state: ReadExpect<'s, MultiplayerGameState>,
    framed_updates: WriteExpect<'s, FramedUpdates<FrameUpdate>>,
    framed_client_side_actions: WriteExpect<'s, FramedUpdates<ClientFrameUpdate>>,
    world_states: WriteExpect<'s, WorldStates>,
    aggregated_outcoming_updates: WriteExpect<'s, AggregatedOutcomingUpdates>,
    entity_net_metadata_storage: WriteExpect<'s, EntityNetMetadataStorage>,
    action_update_id_provider: WriteExpect<'s, ActionUpdateIdProvider>,
    cast_actions_to_execute: WriteExpect<'s, CastActionsToExecute>,
    monster_definitions: ReadExpect<'s, MonsterDefinitions>,
    client_player_actions: ReadStorage<'s, ClientPlayerActions>,
    transforms: WriteStorage<'s, Transform>,
    entity_net_metadata: WriteStorage<'s, EntityNetMetadata>,
    players: WriteStorage<'s, Player>,
    player_actions: WriteStorage<'s, PlayerActions>,
    player_last_casted_spells: WriteStorage<'s, PlayerLastCastedSpells>,
    monsters: WriteStorage<'s, Monster>,
    missiles: WriteStorage<'s, Missile>,
    world_positions: WriteStorage<'s, WorldPosition>,
    net_world_positions: WriteStorage<'s, NetWorldPosition>,
    dead: WriteStorage<'s, Dead>,
    damage_histories: WriteStorage<'s, DamageHistory>,
    hidden_propagates: WriteStorage<'s, HiddenPropagate>,
}

#[cfg(feature = "client")]
#[derive(SystemData)]
pub struct GraphicsSystemData<'s> {
    meshes: WriteStorage<'s, Handle<Mesh>>,
    materials: WriteStorage<'s, Handle<Material>>,
}

#[cfg(not(feature = "client"))]
#[derive(SystemData)]
pub struct GraphicsSystemData<'s> {
    _lifetime: PhantomData<&'s ()>,
}

#[cfg(feature = "client")]
#[derive(SystemData)]
pub struct AnimationsSystemData<'s> {
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    named: ReadStorage<'s, Named>,
    pub animation_sets: ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
    animation_control_sets: WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
}

#[cfg(not(feature = "client"))]
#[derive(SystemData)]
pub struct AnimationsSystemData<'s> {
    _lifetime: PhantomData<&'s ()>,
}

pub struct ActionSystem;

impl<'s> System<'s> for ActionSystem {
    type SystemData = ActionSystemData<'s>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        profile_scope!("ActionSystem::run");
        if !system_data.game_state_helper.is_running() {
            return;
        }
        let game_frame_number = system_data.game_time_service.game_frame_number();
        log::trace!("Frame number: {}", game_frame_number);

        let graphics_resource_bundle =
            create_graphics_resource_bundle(system_data.graphics_system_data);
        let animations_resource_bundle =
            create_animations_resource_bundle(system_data.animations_system_data);

        let transforms = Rc::new(RefCell::new(system_data.transforms));
        let entity_net_metadata_storage =
            Rc::new(RefCell::new(system_data.entity_net_metadata_storage));
        let action_update_id_provider =
            Rc::new(RefCell::new(system_data.action_update_id_provider));
        let entity_net_metadata = Rc::new(RefCell::new(system_data.entity_net_metadata));
        let players = Rc::new(RefCell::new(system_data.players));
        let player_actions = Rc::new(RefCell::new(system_data.player_actions));
        let player_last_casted_spells =
            Rc::new(RefCell::new(system_data.player_last_casted_spells));
        let monsters = Rc::new(RefCell::new(system_data.monsters));
        let missiles = Rc::new(RefCell::new(system_data.missiles));
        let cast_actions_to_execute = Rc::new(RefCell::new(system_data.cast_actions_to_execute));
        let world_positions = Rc::new(RefCell::new(system_data.world_positions));
        let net_world_positions = Rc::new(RefCell::new(system_data.net_world_positions));
        let dead = Rc::new(RefCell::new(system_data.dead));
        let damage_histories = Rc::new(RefCell::new(system_data.damage_histories));
        let hidden_propagates = Rc::new(RefCell::new(system_data.hidden_propagates));

        let world_state_subsystem = WorldStateSubsystem {
            entities: &system_data.entities,
            players: players.clone(),
            player_actions: player_actions.clone(),
            player_last_casted_spells: player_last_casted_spells.clone(),
            monsters: monsters.clone(),
            missiles: missiles.clone(),
            world_positions: world_positions.clone(),
            dead: dead.clone(),
        };
        let player_action_subsystem = PlayerActionSubsystem {
            game_time_service: &system_data.game_time_service,
            game_state_helper: &system_data.game_state_helper,
            entities: &system_data.entities,
            game_level_state: &system_data.game_level_state,
            multiplayer_game_state: &system_data.multiplayer_game_state,
            client_player_actions: &system_data.client_player_actions,
            action_update_id_provider: action_update_id_provider.clone(),
            cast_actions_to_execute: cast_actions_to_execute.clone(),
            player_actions: player_actions.clone(),
            player_last_casted_spells: player_last_casted_spells.clone(),
            missiles: missiles.clone(),
            world_positions: world_positions.clone(),
            animations_resource_bundle: &animations_resource_bundle,
        };
        let monster_action_subsystem = MonsterActionSubsystem {
            entities: &system_data.entities,
            game_time_service: &system_data.game_time_service,
            game_state_helper: &system_data.game_state_helper,
            monster_definitions: &system_data.monster_definitions,
            game_level_state: &system_data.game_level_state,
            client_player_actions: &system_data.client_player_actions,
            entity_net_metadata: entity_net_metadata.clone(),
            players: players.clone(),
            world_positions: world_positions.clone(),
            net_world_positions: net_world_positions.clone(),
            damage_histories: damage_histories.clone(),
            animations_resource_bundle: &animations_resource_bundle,
        };
        let missile_factory = MissileFactory::new(
            &system_data.entities,
            transforms.clone(),
            missiles.clone(),
            &graphics_resource_bundle,
        );
        let missile_spawner_subsystem = MissileSpawnerSubsystem {
            game_time_service: &system_data.game_time_service,
            game_state_helper: &system_data.game_state_helper,
            entities: &system_data.entities,
            missile_factory: &missile_factory,
            cast_actions_to_execute: cast_actions_to_execute.clone(),
            monsters: monsters.clone(),
            dead: dead.clone(),
            world_positions: world_positions.clone(),
        };
        let missile_physics_subsystem = MissilePhysicsSubsystem {
            game_time_service: &system_data.game_time_service,
            game_state_helper: &system_data.game_state_helper,
            game_level_state: &system_data.game_level_state,
            entities: &system_data.entities,
            monsters: monsters.clone(),
            missiles: missiles.clone(),
            dead: dead.clone(),
            damage_histories: damage_histories.clone(),
            world_positions: world_positions.clone(),
            hidden_propagates: hidden_propagates.clone(),
        };
        let damage_subsystem = DamageSubsystem {
            game_state_helper: &system_data.game_state_helper,
            entities: &system_data.entities,
            entity_net_metadata_storage: entity_net_metadata_storage.clone(),
            entity_net_metadata: entity_net_metadata.clone(),
            players: players.clone(),
            monsters: monsters.clone(),
            damage_histories: damage_histories.clone(),
            dead: dead.clone(),
            hidden_propagates: hidden_propagates.clone(),
        };

        system_data
            .framed_updates
            .reserve_updates(game_frame_number);
        system_data
            .framed_client_side_actions
            .reserve_updates(game_frame_number);

        // We may update client actions when discarding updates in ClientNetworkSystem, but as
        // we iterate though framed_updates, we should update its oldest_updated_frame as well.
        system_data.framed_updates.oldest_updated_frame = system_data
            .framed_updates
            .oldest_updated_frame
            .min(system_data.framed_client_side_actions.oldest_updated_frame);

        // Add a world state to save the components to, ensure the update is possible.
        system_data
            .world_states
            .add_world_state(SavedWorldState::default());
        system_data
            .world_states
            .check_update_is_possible(&system_data.framed_updates)
            .unwrap_or_else(|err| {
                panic!(
                    "Expected an update to be possible (current frame {}): {:?}",
                    game_frame_number, err
                )
            });

        let oldest_updated_frame = system_data.framed_updates.oldest_updated_frame;

        // Load NetWorldPositions from currently available saved world states.
        let mut framed_net_positions: Vec<Vec<(Entity, NetWorldPosition)>> =
            if system_data.game_state_helper.is_authoritative() {
                Vec::new()
            } else {
                let capacity =
                    system_data.game_time_service.game_frame_number() - oldest_updated_frame + 1;
                let mut framed_net_positions = Vec::with_capacity(capacity as usize);
                let mut world_states_iter = system_data
                    .world_states
                    .states_iter(oldest_updated_frame.saturating_sub(INTERPOLATION_FRAME_DELAY));
                // Filling with empty values as the first INTERPOLATION_FRAME_DELAY frames
                // we have zero data.
                let zero_data_frames = INTERPOLATION_FRAME_DELAY
                    .saturating_sub(oldest_updated_frame)
                    .min(capacity);
                for _ in 0..zero_data_frames {
                    framed_net_positions.push(Vec::new());
                }
                for _ in zero_data_frames..capacity {
                    let world_state = world_states_iter
                        .next()
                        .expect("Expected a world state while loading NetWorldPosition");
                    let net_positions = world_state
                        .world_positions
                        .iter()
                        .cloned()
                        .map(|(entity, world_position)| (entity, world_position.into()))
                        .collect();
                    framed_net_positions.push(net_positions);
                }
                framed_net_positions
            };

        // Load the world state of the oldest updated frame.
        let mut world_states_iter = system_data
            .world_states
            .states_iter_mut(oldest_updated_frame);
        let mut world_state = world_states_iter.next().unwrap_or_else(|| {
            panic!(
                "Expected to store a world state for frame {}",
                oldest_updated_frame,
            )
        });
        world_state_subsystem.load_from_world_state(world_state);

        // Run each updated frame.
        let mut client_side_actions_iter = system_data
            .framed_client_side_actions
            .updates_iter_mut(oldest_updated_frame);
        for frame_updated in system_data.framed_updates.iter_from_oldest_update() {
            // Update no further than a current frame.
            if game_frame_number < frame_updated.frame_number {
                break;
            }
            let client_side_actions = client_side_actions_iter
                .next()
                .expect("Expected a framed client-side action");

            let outcoming_net_updates = outcoming_net_updates_mut(
                &mut system_data.aggregated_outcoming_updates,
                frame_updated.frame_number,
                system_data.game_time_service.game_frame_number(),
            );

            if !system_data.game_state_helper.is_authoritative() {
                SavedWorldState::load_storage_from(
                    &mut *net_world_positions.borrow_mut(),
                    &framed_net_positions
                        [(frame_updated.frame_number - oldest_updated_frame) as usize],
                );
            }

            damage_subsystem.reset_damage_entries(frame_updated.frame_number);

            let dead_entities = dead.borrow();
            // Run player actions.
            let players_net_metadata = entity_net_metadata.borrow();
            for (entity, mut player, player_net_metadata) in
                (&system_data.entities, &mut *players.borrow_mut())
                    .join()
                    .filter(|(entity, _)| {
                        !is_dead(*entity, &*dead_entities, frame_updated.frame_number)
                    })
                    .map(move |(entity, player)| {
                        (entity, player, players_net_metadata.get(entity).cloned())
                    })
                    .collect::<Vec<_>>()
            {
                // Run walk action.
                let net_args = if system_data.multiplayer_game_state.is_playing {
                    let player_net_metadata =
                        player_net_metadata.expect("Expected EntityNetMetadata for a player");
                    let updates =
                        walk_action_update_for_player(&frame_updated, player_net_metadata);

                    Some(ApplyWalkActionNetArgs {
                        entity_net_id: player_net_metadata.id,
                        outcoming_net_updates,
                        updates,
                    })
                } else {
                    None
                };
                player_action_subsystem.apply_walk_action(
                    frame_updated.frame_number,
                    entity,
                    &mut player,
                    net_args,
                    client_side_actions,
                );

                // Run look action.
                let net_args = if system_data.multiplayer_game_state.is_playing {
                    let player_net_metadata =
                        player_net_metadata.expect("Expected EntityNetMetadata for a player");
                    let updates =
                        look_action_update_for_player(&frame_updated, player_net_metadata);

                    Some(ApplyLookActionNetArgs {
                        entity_net_id: player_net_metadata.id,
                        outcoming_net_updates,
                        update: updates,
                    })
                } else {
                    None
                };
                player_action_subsystem.apply_look_action(
                    frame_updated.frame_number,
                    entity,
                    &mut player,
                    net_args,
                    client_side_actions,
                );

                // Run cast action.
                let net_args = if system_data.multiplayer_game_state.is_playing {
                    let player_net_metadata =
                        player_net_metadata.expect("Expected EntityNetMetadata for a player");
                    let update = cast_action_update_for_player(&frame_updated, player_net_metadata);

                    Some(ApplyCastActionNetArgs {
                        entity_net_id: player_net_metadata.id,
                        outcoming_net_updates,
                        update,
                    })
                } else {
                    None
                };
                player_action_subsystem.apply_cast_action(
                    frame_updated.frame_number,
                    entity,
                    net_args,
                    client_side_actions,
                );
            }

            // Run mob actions.
            let entity_net_metadata_storage = entity_net_metadata_storage.borrow();
            let monsters_net_metadata = entity_net_metadata.borrow();
            for (entity, mut monster, monster_net_metadata) in
                (&system_data.entities, &mut *monsters.borrow_mut())
                    .join()
                    .filter(|(entity, _)| {
                        !is_dead(*entity, &*dead_entities, frame_updated.frame_number)
                    })
                    .map(move |(entity, monster)| {
                        (entity, monster, monsters_net_metadata.get(entity).cloned())
                    })
                    .collect::<Vec<_>>()
            {
                let monster_is_spawned = monster_net_metadata
                    .map(|net_metadata| {
                        net_metadata.spawned_frame_number <= frame_updated.frame_number
                    })
                    .unwrap_or(true);
                if monster_is_spawned {
                    let net_args = if system_data.multiplayer_game_state.is_playing {
                        let monster_net_metadata =
                            monster_net_metadata.expect("Expected EntityNetMetadata for a monster");
                        let updates = mob_actions_update(
                            &frame_updated,
                            monster_net_metadata,
                            &entity_net_metadata_storage,
                        );

                        Some(ApplyMonsterActionNetArgs {
                            entity_net_id: monster_net_metadata.id,
                            outcoming_net_updates,
                            updates,
                        })
                    } else {
                        None
                    };

                    monster_action_subsystem.decide_monster_action(
                        frame_updated.frame_number,
                        entity,
                        &mut monster,
                        net_args,
                    );
                    monster_action_subsystem.process_monster_movement(entity, &mut monster);
                }
            }
            drop(dead_entities);
            drop(entity_net_metadata_storage);

            // Run missile systems.
            missile_spawner_subsystem.spawn_missiles(frame_updated.frame_number);
            missile_physics_subsystem.process_physics(frame_updated.frame_number);

            // Process damage history and add updates, if server.
            damage_subsystem.process_damage_history(
                frame_updated.frame_number,
                damage_histories_updates(&frame_updated),
                outcoming_net_updates,
            );

            // Get the next world state and save the current world to it.
            world_state = world_states_iter.next().unwrap_or_else(|| {
                panic!(
                    "Expected to store a world state for frame {}",
                    frame_updated.frame_number,
                )
            });
            world_state_subsystem.save_world_state(world_state);

            // Update net_positions if we're updating more than INTERPOLATION_FRAME_DELAY frames.
            if frame_updated.frame_number - oldest_updated_frame >= INTERPOLATION_FRAME_DELAY
                && !system_data.game_state_helper.is_authoritative()
            {
                let i =
                    frame_updated.frame_number - oldest_updated_frame - INTERPOLATION_FRAME_DELAY;
                framed_net_positions[i as usize] = world_state
                    .world_positions
                    .iter()
                    .cloned()
                    .map(|(entity, world_position)| (entity, world_position.into()))
                    .collect();
            }
        }

        drop(client_side_actions_iter);
        system_data.framed_updates.oldest_updated_frame = game_frame_number + 1;
        system_data.framed_client_side_actions.oldest_updated_frame = game_frame_number + 1;
    }
}

#[cfg(feature = "client")]
fn create_graphics_resource_bundle(system_data: GraphicsSystemData) -> GraphicsResourceBundle {
    GraphicsResourceBundle {
        meshes: Rc::new(RefCell::new(system_data.meshes)),
        materials: Rc::new(RefCell::new(system_data.materials)),
    }
}

#[cfg(not(feature = "client"))]
fn create_graphics_resource_bundle(_system_data: GraphicsSystemData) -> GraphicsResourceBundle {
    GraphicsResourceBundle {
        _lifetime: PhantomData,
    }
}

#[cfg(feature = "client")]
fn create_animations_resource_bundle(
    system_data: AnimationsSystemData,
) -> AnimationsResourceBundle {
    AnimationsResourceBundle {
        parent_hierarchy: system_data.parent_hierarchy,
        named: system_data.named,
        animation_sets: system_data.animation_sets,
        animation_control_sets: Rc::new(RefCell::new(system_data.animation_control_sets)),
    }
}

#[cfg(not(feature = "client"))]
fn create_animations_resource_bundle(
    _system_data: AnimationsSystemData,
) -> AnimationsResourceBundle {
    AnimationsResourceBundle {
        _lifetime: PhantomData,
    }
}

#[cfg(feature = "client")]
fn damage_histories_updates(
    frame_updates: &FrameUpdate,
) -> Option<&Vec<NetUpdate<DamageHistoryEntries>>> {
    Some(&frame_updates.damage_histories_updates)
}

#[cfg(not(feature = "client"))]
fn damage_histories_updates(
    _frame_updates: &FrameUpdate,
) -> Option<&Vec<NetUpdate<DamageHistoryEntries>>> {
    None
}

#[cfg(feature = "client")]
fn walk_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<(Option<WorldPosition>, ClientActionUpdate<PlayerWalkAction>)> {
    frame_updates
        .player_updates
        .player_walk_actions_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .or_else(|| {
            frame_updates
                .controlled_player_updates
                .player_walk_actions_updates
                .iter()
                .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        })
        .map(move |update| (Some(update.position.clone()), update.data.clone()))
}

#[cfg(not(feature = "client"))]
fn walk_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<(Option<WorldPosition>, ClientActionUpdate<PlayerWalkAction>)> {
    frame_updates
        .walk_action_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| (None, update.data.clone()))
}

#[cfg(feature = "client")]
fn look_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<ClientActionUpdate<PlayerLookAction>> {
    frame_updates
        .player_updates
        .player_look_actions_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| update.data.clone())
}

#[cfg(not(feature = "client"))]
fn look_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<ClientActionUpdate<PlayerLookAction>> {
    frame_updates
        .look_action_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| update.data.clone())
}

#[cfg(feature = "client")]
fn cast_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<IdentifiableAction<ClientActionUpdate<PlayerCastAction>>> {
    frame_updates
        .player_updates
        .player_cast_actions_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .or_else(|| {
            frame_updates
                .controlled_player_updates
                .player_cast_actions_updates
                .iter()
                .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        })
        .map(move |update| update.data.clone())
}

#[cfg(not(feature = "client"))]
fn cast_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<IdentifiableAction<ClientActionUpdate<PlayerCastAction>>> {
    frame_updates
        .cast_action_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| update.data.clone())
}

#[cfg(feature = "client")]
fn mob_actions_update<'a>(
    frame_updates: &'a FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
    entity_net_metadata_service: &'a EntityNetMetadataStorage,
) -> Option<(WorldPosition, MobAction<Entity>)> {
    frame_updates
        .mob_actions_updates
        .iter()
        .find(|update| update.entity_net_id == entity_net_metadata.id)
        .map(move |update| {
            (
                update.position.clone(),
                update
                    .data
                    .clone()
                    .load_entity_by_net_id(&entity_net_metadata_service),
            )
        })
}

#[cfg(not(feature = "client"))]
fn mob_actions_update<'a>(
    _frame_updates: &'a FrameUpdate,
    _entity_net_metadata: EntityNetMetadata,
    _entity_net_metadata_service: &'a EntityNetMetadataStorage,
) -> Option<(WorldPosition, MobAction<Entity>)> {
    None
}
