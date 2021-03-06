use amethyst::{
    core::Time,
    ecs::{prelude::World, ReadExpect, WriteExpect},
    shred::{ResourceId, SystemData},
};

use std::time::Duration;

use crate::ecs::resources::GameTime;

#[derive(SystemData)]
pub struct GameTimeService<'a> {
    engine_time: ReadExpect<'a, Time>,
    game_time: WriteExpect<'a, GameTime>,
}

impl<'a> GameTimeService<'a> {
    pub fn set_game_start_time(&mut self) {
        self.game_time.level_started_at = self.engine_time.absolute_time();
        self.game_time.started_at_frame_number = self.engine_time.frame_number();
        self.game_time.frames_skipped = 0;
    }

    pub fn engine_time(&self) -> &Time {
        &self.engine_time
    }

    pub fn level_duration(&self) -> Duration {
        let level_duration_secs =
            self.game_frame_number() as f32 * self.engine_time.fixed_seconds();
        Duration::from_secs_f32(level_duration_secs)
    }

    pub fn game_frame_number(&self) -> u64 {
        (self.engine_time.frame_number() - self.game_time.started_at_frame_number)
            .saturating_sub(self.game_time.frames_skipped)
    }

    pub fn game_frame_number_absolute(&self) -> u64 {
        self.engine_time.frame_number() - self.game_time.started_at_frame_number
    }

    pub fn seconds_to_frame(&self, game_frame_number: u64) -> f32 {
        self.seconds_between_frames(self.game_frame_number(), game_frame_number)
    }

    pub fn seconds_to_frame_absolute(&self, game_frame_number_absolute: u64) -> f32 {
        self.seconds_between_frames(
            self.game_frame_number_absolute(),
            game_frame_number_absolute,
        )
    }

    pub fn seconds_between_frames(&self, lhs: u64, rhs: u64) -> f32 {
        (lhs as f32 - rhs as f32).abs() * self.engine_time.fixed_seconds()
    }
}
