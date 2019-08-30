use amethyst::ecs::Entity;
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, ops::Range};

use crate::net::{ConnectionIdentifier, EntityNetIdentifier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerRoomPlayer {
    pub connection_id: ConnectionIdentifier,
    pub entity_net_id: EntityNetIdentifier,
    pub nickname: String,
    pub is_host: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MultiplayerGameState {
    pub is_playing: bool,
    pub players: Vec<MultiplayerRoomPlayer>,
    players_updated: bool,
}

impl MultiplayerGameState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            players: Vec::new(),
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
    range: Range<EntityNetIdentifier>,
    mapping: HashMap<EntityNetIdentifier, Entity>,
}

impl EntityNetMetadataStorage {
    pub fn new() -> Self {
        Self {
            range: 0..EntityNetIdentifier::max_value(),
            mapping: HashMap::new(),
        }
    }

    pub fn get_entity(&self, entity_net_id: EntityNetIdentifier) -> Entity {
        self.mapping[&entity_net_id]
    }

    pub fn register_new_entity(&mut self, entity: Entity) -> EntityNetIdentifier {
        let entity_net_id = self
            .range
            .next()
            .expect("Expected a new EntityNetIdentifier");
        self.mapping.insert(entity_net_id, entity);
        entity_net_id
    }

    pub fn set_net_id(&mut self, entity: Entity, entity_net_id: EntityNetIdentifier) {
        self.mapping.insert(entity_net_id, entity);
    }

    pub fn reset(&mut self) {
        self.range = 0..EntityNetIdentifier::max_value();
        self.mapping.clear();
    }
}

impl Default for EntityNetMetadataStorage {
    fn default() -> Self {
        Self::new()
    }
}
