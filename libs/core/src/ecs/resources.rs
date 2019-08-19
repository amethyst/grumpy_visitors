use serde_derive::{Deserialize, Serialize};

use std::time::Duration;

use crate::math::Vector2;

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
    pub nickname: String,
    pub is_host: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MultiplayerRoomPlayers {
    pub players: Vec<MultiplayerRoomPlayer>,
    updated: bool,
}

impl MultiplayerRoomPlayers {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            updated: false,
        }
    }

    pub fn read_updated(&mut self) -> Option<&[MultiplayerRoomPlayer]> {
        if self.updated {
            self.updated = false;
            Some(&self.players)
        } else {
            None
        }
    }

    pub fn update(&mut self) -> &mut Vec<MultiplayerRoomPlayer> {
        self.updated = true;
        &mut self.players
    }
}
