use amethyst::ecs::Entity;
use serde_derive::{Deserialize, Serialize};

use std::collections::{BTreeMap, VecDeque};

use crate::{
    actions::{mob::MobAction, player::PlayerLookAction, Action},
    ecs::components::{
        damage_history::DamageHistoryEntries, missile::Missile, Monster, Player, PlayerActions,
        WorldPosition,
    },
    net::{EntityNetIdentifier, MergableNetUpdates, NetUpdate, NetUpdateWithPosition},
};

const SAVED_WORLD_STATES_LIMIT: usize = 600;

pub struct WorldStates {
    world_states: VecDeque<SavedWorldState>,
}

impl WorldStates {
    pub fn add_world_state(&mut self, world_state: SavedWorldState) {
        self.world_states.push_back(world_state);
        if self.world_states.len() > SAVED_WORLD_STATES_LIMIT {
            self.world_states.pop_front();
        }
    }

    pub fn states_iter_mut(
        &mut self,
        start_frame_number: u64,
    ) -> impl Iterator<Item = &mut SavedWorldState> {
        self.world_states
            .iter_mut()
            .skip_while(move |world_state| world_state.frame_number < start_frame_number)
    }

    pub fn can_apply_updates<T>(&self, framed_updates: &FramedUpdates<T>) -> bool {
        self.world_states
            .front()
            .map(|state| state.frame_number == framed_updates.oldest_updated_frame)
            .unwrap_or(false)
    }
}

impl Default for WorldStates {
    fn default() -> Self {
        let mut world_states = VecDeque::new();
        world_states.push_back(SavedWorldState::default());
        Self { world_states }
    }
}

// TODO: benchmark in order to justify the collection choice (BTreeMap vs HashMap vs Vec).
#[derive(Default)]
pub struct SavedWorldState {
    pub frame_number: u64,
    pub players: BTreeMap<Entity, Player>,
    pub monsters: BTreeMap<Entity, Monster>,
    pub missiles: BTreeMap<Entity, Missile>,
    pub world_positions: BTreeMap<Entity, WorldPosition>,
}

pub struct FramedUpdates<T> {
    pub oldest_updated_frame: u64,
    pub updates: VecDeque<T>,
}

impl<T: FramedUpdate> FramedUpdates<T> {
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
}

impl<T> Default for FramedUpdates<T> {
    fn default() -> Self {
        Self {
            oldest_updated_frame: 0,
            updates: VecDeque::new(),
        }
    }
}

pub trait FramedUpdate {
    fn frame_number(&self) -> u64;
}

pub struct ServerWorldUpdates {
    pub updates: Vec<(u64, ServerWorldUpdate)>,
}

pub struct PlayerActionUpdates {
    pub frame_number: u64,
    pub updates: Vec<NetUpdate<PlayerActions>>,
}

impl FramedUpdate for PlayerActionUpdates {
    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

/// Is sent by client, gets aggregated into PlayerActionUpdates on server side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActionUpdate<T> {
    pub frame_number: u64,
    pub update: NetUpdate<T>,
}

impl<T> FramedUpdate for PlayerActionUpdate<T> {
    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

/// Resource. Is sent by client, gets aggregated into PlayerActionUpdates on server side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLookActionUpdates {
    /// Updates for each player.
    pub updates: Vec<PlayerActionUpdate<PlayerLookAction>>,
}

/// Is sent by server, stored in FramedUpdates<T> by client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerWorldUpdate {
    pub frame_number: u64,
    pub player_actions_updates: Vec<NetUpdateWithPosition<PlayerActions>>,
    pub mob_actions_updates: Vec<NetUpdateWithPosition<Action<MobAction<EntityNetIdentifier>>>>,
    pub damage_histories_updates: Vec<NetUpdate<DamageHistoryEntries>>,
}

impl FramedUpdate for ServerWorldUpdate {
    fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

impl ServerWorldUpdate {
    pub fn merge_another_update(&mut self, other: ServerWorldUpdate) {
        assert_eq!(self.frame_number, other.frame_number);
        self.player_actions_updates
            .merge(other.player_actions_updates);
        self.mob_actions_updates.merge(other.mob_actions_updates);
        self.damage_histories_updates
            .merge(other.damage_histories_updates);
    }
}
