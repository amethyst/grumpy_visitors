pub mod net;
pub mod world;

use std::time::{Duration, Instant};

use crate::math::Vector2;

pub struct GameTime {
    pub level_started_at: Duration,
    pub started_at_frame_number: u64,
    pub frames_skipped: u64,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            level_started_at: Duration::new(0, 0),
            started_at_frame_number: 0,
            frames_skipped: 0,
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
    ShuttingDown { shutdown_at: Instant },
    Quit,
}

impl GameEngineState {
    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }
}

impl NewGameEngineState {
    pub fn shutdown() -> Self {
        NewGameEngineState(GameEngineState::ShuttingDown {
            shutdown_at: Instant::now() + Duration::from_secs(1),
        })
    }
}
