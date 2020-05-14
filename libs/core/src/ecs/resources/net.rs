use amethyst::ecs::Entity;
use derivative::Derivative;
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, ops::Range};

use crate::{
    actions::{player::PlayerCastAction, IdentifiableAction},
    net::{server_message::PlayerNetStatus, NetIdentifier},
    PLAYER_COLORS,
};

#[derive(Derivative, Debug, Clone, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct MultiplayerRoomPlayer {
    pub connection_id: NetIdentifier,
    pub entity_net_id: NetIdentifier,
    pub nickname: String,
    pub is_host: bool,
    #[derivative(PartialEq = "ignore")]
    pub color: [f32; 3],
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MultiplayerGameState {
    pub is_playing: bool,
    pub players: Vec<MultiplayerRoomPlayer>,
    pub waiting_network: bool,
    pub waiting_for_players: bool,
    /// This is used on client to make sure that we do not unpause before pausing.
    pub waiting_for_players_pause_id: u64,
    /// To help keep the track of outdated status reports (they use unreliable channel).
    pub players_status_id: u64,
    pub lagging_players: Vec<NetIdentifier>,
    pub is_disconnected: bool,
    players_updated: bool,
}

impl MultiplayerGameState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            players: Vec::new(),
            waiting_network: false,
            waiting_for_players: false,
            waiting_for_players_pause_id: 0,
            players_status_id: 0,
            lagging_players: Vec::new(),
            is_disconnected: false,
            players_updated: false,
        }
    }

    pub fn reset(&mut self) {
        *self = MultiplayerGameState::new();
    }

    pub fn read_updated_players(&mut self) -> Option<&[MultiplayerRoomPlayer]> {
        if self.players_updated {
            self.players_updated = false;
            Some(&self.players)
        } else {
            None
        }
    }

    pub fn update_players(&mut self) -> &mut Vec<MultiplayerRoomPlayer> {
        self.players_updated = true;
        &mut self.players
    }

    pub fn drop_player_by_connection_id(&mut self, player_connection_id: NetIdentifier) {
        let player_index = self
            .players
            .iter()
            .position(|player| player.connection_id == player_connection_id);
        if let Some(player_index) = player_index {
            self.drop_player_by_index(player_index);
        } else {
            log::warn!(
                "Couldn't find a player with connection id {} to drop",
                player_connection_id
            );
        }
    }

    pub fn drop_player_by_index(&mut self, player_index: usize) {
        self.players_updated = true;
        self.players.remove(player_index);
        for (player_index, player) in self.players.iter_mut().enumerate().skip(player_index) {
            player.color = PLAYER_COLORS[player_index];
        }
    }

    pub fn find_player_by_connection_id(
        &self,
        player_connection_id: NetIdentifier,
    ) -> Option<&MultiplayerRoomPlayer> {
        self.players
            .iter()
            .find(|player| player.connection_id == player_connection_id)
    }
}

pub struct EntityNetMetadataStorage {
    range: Range<NetIdentifier>,
    mapping: HashMap<NetIdentifier, Entity>,
}

impl EntityNetMetadataStorage {
    pub fn new() -> Self {
        Self {
            range: 0..NetIdentifier::max_value(),
            mapping: HashMap::new(),
        }
    }

    pub fn get_entity(&self, entity_net_id: NetIdentifier) -> Option<Entity> {
        self.mapping.get(&entity_net_id).cloned()
    }

    pub fn reserve_ids(&mut self, count: usize) -> Range<NetIdentifier> {
        let start = self.range.start;
        self.range.start += count as NetIdentifier;
        start..self.range.start
    }

    pub fn register_new_entity(&mut self, entity: Entity) -> NetIdentifier {
        let entity_net_id = self
            .range
            .next()
            .expect("Expected a new EntityNetIdentifier");
        self.mapping.insert(entity_net_id, entity);
        entity_net_id
    }

    pub fn set_net_id(&mut self, entity: Entity, entity_net_id: NetIdentifier) {
        self.mapping.insert(entity_net_id, entity);
    }

    pub fn reset(&mut self) {
        self.range = 0..NetIdentifier::max_value();
        self.mapping.clear();
    }
}

impl Default for EntityNetMetadataStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct ActionUpdateIdProvider {
    update_id_autoinc: NetIdentifier,
}

impl ActionUpdateIdProvider {
    pub fn next_update_id(&mut self) -> NetIdentifier {
        let id = self.update_id_autoinc;
        self.update_id_autoinc = self.update_id_autoinc.wrapping_add(1);
        id
    }
}

#[derive(Default)]
pub struct CastActionsToExecute {
    pub actions: Vec<IdentifiableAction<PlayerCastAction>>,
}

#[derive(Default)]
pub struct PlayersNetStatus {
    pub frame_received: u64,
    pub players: Vec<PlayerNetStatus>,
}
