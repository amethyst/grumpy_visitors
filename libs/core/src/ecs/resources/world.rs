use amethyst::ecs::{Component, Entities, Entity, Join, ReadStorage, WriteStorage};
use serde_derive::{Deserialize, Serialize};

use std::{collections::VecDeque, iter::FromIterator};

use crate::{
    actions::{
        mob::MobAction,
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
        ClientActionUpdate,
    },
    ecs::components::{
        damage_history::DamageHistoryEntries, missile::Missile, Dead, Monster, Player,
        PlayerActions, WorldPosition,
    },
    net::{NetIdentifier, NetUpdate, NetUpdateWithPosition},
};

const SAVED_WORLD_STATES_LIMIT: usize = 600;
pub const LAG_COMPENSATION_FRAMES_LIMIT: usize = 20;

#[derive(Debug)]
pub struct OldFrameError {
    last_available_frame: Option<u64>,
    requested_frame: u64,
}

pub struct WorldStates {
    world_states: VecDeque<SavedWorldState>,
}

impl WorldStates {
    pub fn add_world_state(&mut self, mut world_state: SavedWorldState) {
        if self.world_states.is_empty() {
            world_state.frame_number = 0;
        } else {
            world_state.frame_number = self.world_states.back().unwrap().frame_number + 1;
        }

        self.world_states.push_back(world_state);
        if self.world_states.len() > SAVED_WORLD_STATES_LIMIT {
            self.world_states.pop_front();
        }
    }

    pub fn states_iter_mut(
        &mut self,
        start_frame_number: u64,
    ) -> impl Iterator<Item = &mut SavedWorldState> {
        let elements_to_skip = self
            .world_states
            .iter()
            .position(|world_state| world_state.frame_number == start_frame_number)
            .unwrap_or_else(|| self.world_states.len().saturating_sub(1));
        self.world_states.iter_mut().skip(elements_to_skip)
    }

    pub fn states_iter(&self, start_frame_number: u64) -> impl Iterator<Item = &SavedWorldState> {
        self.world_states
            .iter()
            .skip_while(move |world_state| world_state.frame_number < start_frame_number)
    }

    pub fn check_update_is_possible<T>(
        &self,
        framed_updates: &FramedUpdates<T>,
    ) -> Result<(), OldFrameError> {
        if let Some(state) = self.world_states.front() {
            if state.frame_number <= framed_updates.oldest_updated_frame {
                Ok(())
            } else {
                Err(OldFrameError {
                    last_available_frame: Some(state.frame_number),
                    requested_frame: framed_updates.oldest_updated_frame,
                })
            }
        } else {
            Err(OldFrameError {
                last_available_frame: None,
                requested_frame: framed_updates.oldest_updated_frame,
            })
        }
    }
}

impl Default for WorldStates {
    fn default() -> Self {
        let mut world_states = VecDeque::with_capacity(SAVED_WORLD_STATES_LIMIT);
        world_states.push_back(SavedWorldState::default());
        Self { world_states }
    }
}

// TODO: benchmark in order to justify the collection choice (BTreeMap vs HashMap vs Vec).
#[derive(Default)]
pub struct SavedWorldState {
    pub frame_number: u64,
    pub players: Vec<(Entity, Player)>,
    pub player_actions: Vec<(Entity, PlayerActions)>,
    pub monsters: Vec<(Entity, Monster)>,
    pub missiles: Vec<(Entity, Missile)>,
    pub world_positions: Vec<(Entity, WorldPosition)>,
    pub dead: Vec<(Entity, Dead)>,
}

impl SavedWorldState {
    pub fn copy_from_storage<T: Clone + Component>(
        entities: &Entities,
        storage: &ReadStorage<T>,
    ) -> Vec<(Entity, T)> {
        Vec::from_iter(
            (entities, storage)
                .join()
                .map(|(entity, component)| (entity, component.clone())),
        )
    }

    pub fn copy_from_write_storage<T: Clone + Component>(
        entities: &Entities,
        storage: &WriteStorage<T>,
    ) -> Vec<(Entity, T)> {
        Vec::from_iter(
            (entities, storage)
                .join()
                .map(|(entity, component)| (entity, component.clone())),
        )
    }

    pub fn load_storage_from<T: Clone + Component>(
        storage: &mut WriteStorage<T>,
        saved_components: &[(Entity, T)],
    ) {
        for (entity, component) in saved_components {
            storage
                .insert(entity.clone(), component.clone())
                .expect("Expected to insert a saved component");
        }
    }
}

pub struct FramedUpdates<T> {
    pub oldest_updated_frame: u64,
    pub updates: VecDeque<T>,
}

impl<T: FramedUpdate + ::std::fmt::Debug> FramedUpdates<T> {
    pub fn reserve_updates(&mut self, frame_number: u64) {
        if frame_number == 0 && self.updates.is_empty() {
            self.add_update();
            return;
        }

        let frames_to_add = frame_number.saturating_sub(self.latest_frame());
        for _ in 0..frames_to_add {
            self.add_update();
        }
    }

    /// Gets a mutable frame, optionally taking into account lag compensation.
    /// If `lag_compensate` parameter equals `true` and the requested frame is too old,
    /// get the first one that is appropriate.
    ///
    /// Updates `oldest_updated_frame`.
    ///
    /// Returns `None` if the `frame_number` passed is from the future.
    pub fn update_frame(&mut self, frame_number: u64, lag_compensate: bool) -> Option<&mut T> {
        self.reserve_updates(frame_number);

        let latest_frame = self.latest_frame();
        let available_frames_count = self.updates.len().min(LAG_COMPENSATION_FRAMES_LIMIT);
        let frames_to_skip = self.updates.len() - available_frames_count;

        let mut iter = self.updates.iter_mut();
        let update_finder = |update: &mut T| update.frame_number() == frame_number;
        let update_index = if lag_compensate {
            iter.skip(frames_to_skip).position(update_finder)
        } else {
            iter.position(update_finder)
        };

        let update_index = if update_index.is_none() && lag_compensate {
            log::debug!(
                "Lag compensating while updating frame {}: skip {} frames",
                frame_number,
                frames_to_skip
            );
            Some(frames_to_skip)
        } else {
            update_index.map(|index| index + frames_to_skip)
        };
        let update = update_index
            .and_then(move |index| {
                let update_frame_number = self
                    .updates
                    .get(index)
                    .unwrap_or_else(|| {
                        panic!(
                            "Expected to find an update for {} frame (latest frame update: {})",
                            frame_number, latest_frame
                        )
                    })
                    .frame_number();
                if update_frame_number < self.oldest_updated_frame {
                    self.oldest_updated_frame = update_frame_number;
                }
                self.updates.get_mut(index)
            })
            .unwrap();

        Some(update)
    }

    pub fn updates_iter_mut(&mut self, start_frame_number: u64) -> impl Iterator<Item = &mut T> {
        self.updates
            .iter_mut()
            .skip_while(move |update| update.frame_number() < start_frame_number)
    }

    pub fn iter_from_oldest_update(&self) -> impl Iterator<Item = &T> {
        let oldest_updated_frame = self.oldest_updated_frame;
        self.updates
            .iter()
            .skip_while(move |update| update.frame_number() < oldest_updated_frame)
    }

    pub fn latest_frame(&self) -> u64 {
        if self.updates.is_empty() {
            0
        } else {
            self.updates.back().unwrap().frame_number()
        }
    }

    fn next_frame(&self) -> u64 {
        if self.updates.is_empty() {
            0
        } else {
            self.latest_frame() + 1
        }
    }

    fn add_update(&mut self) {
        if self.updates.len() == SAVED_WORLD_STATES_LIMIT {
            let removed_update = self.updates.pop_front().unwrap();
            if removed_update.frame_number() == self.oldest_updated_frame {
                self.oldest_updated_frame += 1;
            }
        }

        let update = T::new_update(self.next_frame());
        self.updates.push_back(update);
    }
}

impl<T> Default for FramedUpdates<T> {
    fn default() -> Self {
        Self {
            oldest_updated_frame: 0,
            updates: VecDeque::with_capacity(SAVED_WORLD_STATES_LIMIT),
        }
    }
}

pub trait FramedUpdate {
    fn new_update(frame_number: u64) -> Self;

    fn frame_number(&self) -> u64;
}

/// The resource which aggregates all the updates a server is going to broadcast.
#[derive(Default)]
pub struct ServerWorldUpdates {
    pub updates: VecDeque<(u64, ServerWorldUpdate)>,
}

impl ServerWorldUpdates {
    pub fn create_new_update(&mut self, frame_number: u64) -> &mut ServerWorldUpdate {
        let new_update_id = if self.updates.is_empty() {
            0
        } else {
            let latest_update = &self.updates.back().unwrap();
            latest_update.0 + 1
        };
        self.updates
            .push_back((new_update_id, ServerWorldUpdate::new(frame_number)));
        &mut self.updates.back_mut().unwrap().1
    }
}

/// The resource which aggregates all the updates a client is going to broadcast.
#[derive(Default)]
pub struct ClientWorldUpdates {
    /// Immediate update.
    pub walk_action_updates: Vec<NetUpdate<ClientActionUpdate<PlayerWalkAction>>>,
    /// Immediate update.
    pub cast_action_updates: Vec<NetUpdate<ClientActionUpdate<PlayerCastAction>>>,
    /// Batched update.
    pub look_actions_updates: VecDeque<(u64, Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>)>,
}

/// Both client and server side framed update.
/// Client uses it to store the updates until it receives their confirmations from a server.
/// Server uses it as the main resource of client updates, stores SAVED_WORLD_STATES_LIMIT of them.
#[derive(Debug)]
pub struct PlayerActionUpdates {
    pub frame_number: u64,
    pub walk_action_updates: Vec<NetUpdate<ClientActionUpdate<PlayerWalkAction>>>,
    pub cast_action_updates: Vec<NetUpdate<ClientActionUpdate<PlayerCastAction>>>,
    pub look_action_updates: Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>,
}

impl PlayerActionUpdates {
    pub fn add_walk_action_updates(
        &mut self,
        mut action_update: ImmediatePlayerActionsUpdates<PlayerWalkAction>,
    ) {
        assert_eq!(self.frame_number, action_update.frame_number);
        self.walk_action_updates.append(&mut action_update.updates);
    }

    pub fn add_cast_action_updates(
        &mut self,
        mut action_update: ImmediatePlayerActionsUpdates<PlayerCastAction>,
    ) {
        assert_eq!(self.frame_number, action_update.frame_number);
        self.cast_action_updates.append(&mut action_update.updates);
    }

    pub fn add_look_action_updates(
        &mut self,
        action_updates: (u64, Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>),
    ) {
        let (frame_number, mut action_updates) = action_updates;
        assert_eq!(self.frame_number, frame_number);
        self.look_action_updates.append(&mut action_updates);
    }
}

impl FramedUpdate for PlayerActionUpdates {
    fn new_update(frame_number: u64) -> Self {
        Self {
            frame_number,
            walk_action_updates: Vec::new(),
            cast_action_updates: Vec::new(),
            look_action_updates: Vec::new(),
        }
    }

    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

/// Is sent by client, gets aggregated into PlayerActionUpdates on server side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmediatePlayerActionsUpdates<T> {
    pub frame_number: u64,
    pub updates: Vec<NetUpdate<ClientActionUpdate<T>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLookActionUpdates {
    /// Updates for each player.
    pub updates: Vec<(u64, Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>)>,
}

/// Is sent by server, stored in FramedUpdates<T> by client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerWorldUpdate {
    pub frame_number: u64,
    pub player_walk_actions_updates:
        Vec<NetUpdateWithPosition<ClientActionUpdate<PlayerWalkAction>>>,
    pub player_look_actions_updates: Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>,
    pub player_cast_actions_updates: Vec<NetUpdate<ClientActionUpdate<PlayerCastAction>>>,
    pub mob_actions_updates: Vec<NetUpdateWithPosition<MobAction<NetIdentifier>>>,
    pub damage_histories_updates: Vec<NetUpdate<DamageHistoryEntries>>,
}

impl ServerWorldUpdate {
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            player_walk_actions_updates: Vec::new(),
            player_look_actions_updates: Vec::new(),
            player_cast_actions_updates: Vec::new(),
            mob_actions_updates: Vec::new(),
            damage_histories_updates: Vec::new(),
        }
    }
}

/// I hate this struct name.
#[derive(Debug, Clone)]
pub struct ReceivedServerWorldUpdate {
    pub frame_number: u64,
    pub player_updates: ReceivedPlayerUpdate,
    pub controlled_player_updates: ReceivedPlayerUpdate,
    pub mob_actions_updates: Vec<NetUpdateWithPosition<MobAction<NetIdentifier>>>,
    pub damage_histories_updates: Vec<NetUpdate<DamageHistoryEntries>>,
}

impl ReceivedServerWorldUpdate {
    pub fn apply_server_update(&mut self, server_update: ServerWorldUpdate) {
        self.player_updates.player_walk_actions_updates = server_update.player_walk_actions_updates;
        self.player_updates.player_look_actions_updates = server_update.player_look_actions_updates;
        self.player_updates.player_cast_actions_updates = server_update.player_cast_actions_updates;
        self.mob_actions_updates = server_update.mob_actions_updates;
        self.damage_histories_updates = server_update.damage_histories_updates;
    }
}

#[derive(Default, Debug, Clone)]
pub struct ReceivedPlayerUpdate {
    pub player_walk_actions_updates:
        Vec<NetUpdateWithPosition<ClientActionUpdate<PlayerWalkAction>>>,
    pub player_look_actions_updates: Vec<NetUpdate<ClientActionUpdate<PlayerLookAction>>>,
    pub player_cast_actions_updates: Vec<NetUpdate<ClientActionUpdate<PlayerCastAction>>>,
}

impl FramedUpdate for ReceivedServerWorldUpdate {
    fn new_update(frame_number: u64) -> Self {
        Self {
            frame_number,
            player_updates: ReceivedPlayerUpdate::default(),
            controlled_player_updates: ReceivedPlayerUpdate::default(),
            mob_actions_updates: Vec::new(),
            damage_histories_updates: Vec::new(),
        }
    }

    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

#[derive(Debug)]
pub struct DummyFramedUpdate {
    pub frame_number: u64,
}

impl FramedUpdate for DummyFramedUpdate {
    fn new_update(frame_number: u64) -> Self {
        Self { frame_number }
    }

    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}
