use amethyst::{
    core::Time,
    ecs::{ReadExpect, WriteExpect},
};
use shred_derive::SystemData;

use std::time::Duration;

use crate::data_resources::GameTime;

#[derive(SystemData)]
pub struct GameTimeService<'a> {
    engine_time: ReadExpect<'a, Time>,
    game_time: WriteExpect<'a, GameTime>,
}

impl<'a> GameTimeService<'a> {
    pub fn set_level_started_at(&mut self) {
        self.game_time.level_started_at = self.engine_time.absolute_time();
    }

    pub fn engine_time(&self) -> &Time {
        &self.engine_time
    }

    pub fn level_duration(&self) -> Duration {
        self.engine_time.absolute_time() - self.game_time.level_started_at
    }
}
