use amethyst::ecs::Entity;
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, ops::Range};

use crate::net::NetIdentifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerRoomPlayer {
    pub connection_id: NetIdentifier,
    pub entity_net_id: NetIdentifier,
    pub nickname: String,
    pub is_host: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MultiplayerGameState {
    pub is_playing: bool,
    pub players: Vec<MultiplayerRoomPlayer>,
    pub waiting_network: bool,
    pub waiting_for_players: bool,
    /// This is used on client to make sure that we do not unpause before pausing.
    pub waiting_for_players_pause_id: u64,
    pub lagging_players: Vec<NetIdentifier>,
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
            lagging_players: Vec::new(),
            players_updated: false,
        }
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

    pub fn get_entity(&self, entity_net_id: NetIdentifier) -> Entity {
        self.mapping[&entity_net_id]
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
