use amethyst::ecs::Entity;
use serde_derive::{Deserialize, Serialize};

use std::{collections::HashMap, ops::Range, time::Duration};

use crate::{
    math::Vector2,
    net::{ConnectionIdentifier, EntityNetIdentifier},
};

pub struct GameTime {
    pub level_started_at: Duration,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            level_started_at: Duration::new(0, 0),
        }
    }
}

pub struct GameLevelState {
    pub dimensions: Vector2,
    pub is_over: bool,
    pub spawn_level: usize,
    pub spawn_level_started: Duration,
    pub last_borderline_spawn: Duration,
    pub last_random_spawn: Duration,
}

impl GameLevelState {
    pub fn dimensions_half_size(&self) -> Vector2 {
        self.dimensions / 2.0
    }
}

impl Default for GameLevelState {
    fn default() -> Self {
        Self {
            dimensions: Vector2::new(4096.0, 4096.0),
            is_over: false,
            spawn_level: 1,
            spawn_level_started: Duration::new(0, 0),
            last_borderline_spawn: Duration::new(0, 0),
            last_random_spawn: Duration::new(0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NewGameEngineState(pub GameEngineState);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GameEngineState {
    Loading,
    Menu,
    Playing,
    Quit,
}

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

pub struct EntityNetMetadataService {
    range: Range<EntityNetIdentifier>,
    mapping: HashMap<EntityNetIdentifier, Entity>,
}

impl EntityNetMetadataService {
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

impl Default for EntityNetMetadataService {
    fn default() -> Self {
        Self::new()
    }
}
