use amethyst::ecs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "client")]
use ha_core::ecs::resources::world::{ClientWorldUpdates, ServerWorldUpdate};
#[cfg(not(feature = "client"))]
use ha_core::ecs::resources::world::{PlayerActionUpdates, ServerWorldUpdates};
use ha_core::{
    actions::player::PlayerWalkAction,
    ecs::{
        components::{
            damage_history::DamageHistory, missile::Missile, ClientPlayerActions, Dead,
            EntityNetMetadata, Monster, Player, PlayerActions, WorldPosition,
        },
        resources::{
            net::{EntityNetMetadataStorage, MultiplayerGameState},
            world::{FramedUpdates, SavedWorldState, WorldStates},
            GameEngineState, GameLevelState,
        },
        system_data::time::GameTimeService,
    },
};

use crate::ecs::{
    resources::MonsterDefinitions,
    systems::{
        monster::MonsterActionSubsystem,
        player::{ApplyWalkActionNetArgs, PlayerActionSubsystem},
        world_state_subsystem::WorldStateSubsystem,
        OutcomingNetUpdates,
    },
};

#[cfg(feature = "client")]
type FrameUpdate = ServerWorldUpdate;
#[cfg(not(feature = "client"))]
type FrameUpdate = PlayerActionUpdates;

#[cfg(feature = "client")]
type AggregatedOutcomingUpdates = ClientWorldUpdates;
#[cfg(not(feature = "client"))]
type AggregatedOutcomingUpdates = ServerWorldUpdates;

pub struct ActionSystem;

impl<'s> System<'s> for ActionSystem {
    type SystemData = (
        Entities<'s>,
        GameTimeService<'s>,
        ReadExpect<'s, GameEngineState>,
        ReadExpect<'s, GameLevelState>,
        ReadExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, FramedUpdates<FrameUpdate>>,
        WriteExpect<'s, WorldStates>,
        WriteExpect<'s, AggregatedOutcomingUpdates>,
        ReadExpect<'s, EntityNetMetadataStorage>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadStorage<'s, EntityNetMetadata>,
        ReadStorage<'s, ClientPlayerActions>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, PlayerActions>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, Missile>,
        WriteStorage<'s, WorldPosition>,
        WriteStorage<'s, Dead>,
        WriteStorage<'s, DamageHistory>,
    );

    fn run(
        &mut self,
        (
            entities,
            game_time_service,
            game_engine_state,
            game_level_state,
            multiplayer_game_state,
            mut framed_updates,
            mut world_states,
            mut aggregated_outcoming_updates,
            entity_net_metadata_service,
            monster_definitions,
            entity_net_metadata,
            client_player_actions,
            players,
            player_actions,
            monsters,
            missiles,
            world_positions,
            dead,
            damage_histories,
        ): Self::SystemData,
    ) {
        if *game_engine_state != GameEngineState::Playing {
            return;
        }
        log::trace!("Frame number: {}", game_time_service.game_frame_number());

        let players = Rc::new(RefCell::new(players));
        let player_actions = Rc::new(RefCell::new(player_actions));
        let monsters = Rc::new(RefCell::new(monsters));
        let missiles = Rc::new(RefCell::new(missiles));
        let world_positions = Rc::new(RefCell::new(world_positions));
        let dead = Rc::new(RefCell::new(dead));
        let damage_histories = Rc::new(RefCell::new(damage_histories));

        let world_state_subsystem = WorldStateSubsystem {
            entities: &entities,
            players: players.clone(),
            player_actions: player_actions.clone(),
            monsters: monsters.clone(),
            missiles: missiles.clone(),
            world_positions: world_positions.clone(),
            dead: dead.clone(),
        };
        let player_action_subsystem = PlayerActionSubsystem {
            game_time_service: &game_time_service,
            game_level_state: &game_level_state,
            multiplayer_game_state: &multiplayer_game_state,
            entity_net_metadata_service: &entity_net_metadata_service,
            client_player_actions: &client_player_actions,
            player_actions: player_actions.clone(),
            world_positions: world_positions.clone(),
        };
        let _monster_action_subsystem = MonsterActionSubsystem {
            entities: &entities,
            game_time_service: &game_time_service,
            monster_definitions: &monster_definitions,
            game_level_state: &game_level_state,
            players: players.clone(),
            world_positions: world_positions.clone(),
            damage_histories: damage_histories.clone(),
            monsters: monsters.clone(),
        };

        framed_updates.reserve_updates(game_time_service.game_frame_number());

        // Add a world state to save the components to, insure the update is possible.
        world_states.add_world_state(SavedWorldState::default());
        world_states
            .check_update_is_possible(&framed_updates)
            .unwrap();

        // Load the world state of the oldest updated frame.
        let mut world_states_iter =
            world_states.states_iter_mut(framed_updates.oldest_updated_frame);
        let mut world_state = world_states_iter.next().unwrap_or_else(|| {
            panic!(
                "Expected to store a world state for frame {}",
                framed_updates.oldest_updated_frame,
            )
        });
        world_state_subsystem.load_from_world_state(world_state);

        // Run each updated frame.
        for frame_updated in framed_updates.iter_from_oldest_update() {
            // Update no further than a current frame.
            if game_time_service.game_frame_number() < frame_updated.frame_number {
                break;
            }

            let outcoming_net_updates = outcoming_net_updates_mut(
                &mut aggregated_outcoming_updates,
                frame_updated.frame_number,
            );

            // Run player actions.
            for (entity, mut player, ()) in
                (&entities, &mut *players.borrow_mut(), !&*dead.borrow()).join()
            {
                let net_args = if multiplayer_game_state.is_playing {
                    let player_net_metadata = entity_net_metadata
                        .get(entity)
                        .expect("Expected EntityNetMetadata for a player");

                    let updates =
                        walk_action_update_for_player(&frame_updated, *player_net_metadata);

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
                );
            }

            // Run mob actions.
            //for (entity, mob_position, mob_action) in
            //    mob_actions_updates(&frame_updated, &entity_net_metadata_service)
            //{
            //    monster_action_subsystem.decide_monster_action(
            //        entity,
            //        &mob_position,
            //        &mob_action,
            //        frame_updated.frame_number,
            //    );
            //    monster_action_subsystem.process_monster_movement(entity);
            //}

            // TODO: Run missiles.

            // Get the next world state and save the current world to it..
            world_state = world_states_iter.next().unwrap_or_else(|| {
                panic!(
                    "Expected to store a world state for frame {}",
                    frame_updated.frame_number,
                )
            });
            world_state_subsystem.save_world_state(world_state);
        }

        framed_updates.oldest_updated_frame = game_time_service.game_frame_number();
    }
}

#[cfg(feature = "client")]
fn outcoming_net_updates_mut(
    aggregated_updates: &mut AggregatedOutcomingUpdates,
    _frame_number: u64,
) -> &mut OutcomingNetUpdates {
    aggregated_updates
}

#[cfg(not(feature = "client"))]
fn outcoming_net_updates_mut(
    aggregated_updates: &mut AggregatedOutcomingUpdates,
    frame_number: u64,
) -> &mut OutcomingNetUpdates {
    aggregated_updates.create_new_update(frame_number)
}

#[cfg(feature = "client")]
fn walk_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<(Option<WorldPosition>, Option<PlayerWalkAction>)> {
    frame_updates
        .player_walk_actions_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| (Some(update.position.clone()), update.data.clone()))
}

#[cfg(not(feature = "client"))]
fn walk_action_update_for_player(
    frame_updates: &FrameUpdate,
    entity_net_metadata: EntityNetMetadata,
) -> Option<(Option<WorldPosition>, Option<PlayerWalkAction>)> {
    frame_updates
        .walk_action_updates
        .iter()
        .find(|actions_updates| actions_updates.entity_net_id == entity_net_metadata.id)
        .map(move |update| (None, update.data.clone()))
}

//#[cfg(feature = "client")]
//fn mob_actions_updates<'a>(
//    frame_updates: &'a FrameUpdate,
//    entity_net_metadata_service: &'a EntityNetMetadataStorage,
//) -> impl Iterator<Item = (Entity, Option<WorldPosition>, MobAction<Entity>)> + 'a {
//    frame_updates.mob_actions_updates.iter().map(move |update| {
//        (
//            entity_net_metadata_service.get_entity(update.entity_net_id),
//            Some(update.position.clone()),
//            update
//                .data
//                .clone()
//                .load_entity_by_net_id(&entity_net_metadata_service),
//        )
//    })
//}
//
//#[cfg(not(feature = "client"))]
//fn mob_actions_updates<'a>(
//    _frame_updates: &'a FrameUpdate,
//    _entity_net_metadata_service: &'a EntityNetMetadataStorage,
//) -> impl Iterator<Item = (Entity, Option<WorldPosition>, MobAction<Entity>)> + 'a {
//    std::iter::empty()
//}
