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
    }

    pub fn engine_time(&self) -> &Time {
        &self.engine_time
    }

    pub fn level_duration(&self) -> Duration {
        (self.engine_time.absolute_time() - self.game_time.level_started_at)
            .checked_sub(
                self.engine_time
                    .fixed_time()
                    .checked_mul(self.game_time.frames_skipped as u32)
                    .expect("Expected to multiply Duration"),
            )
            .unwrap_or_else(|| Duration::new(0, 0))
    }

    pub fn game_frame_number(&self) -> u64 {
        (self.engine_time.frame_number() - self.game_time.started_at_frame_number)
            .saturating_sub(self.game_time.frames_skipped)
    }

    pub fn game_frame_number_absolute(&self) -> u64 {
        self.engine_time.frame_number() - self.game_time.started_at_frame_number
    }

    pub fn seconds_to_frame(&self, frame_number: u64) -> f32 {
        self.seconds_between_frames(
            self.game_time.started_at_frame_number,
            frame_number.saturating_sub(self.game_time.frames_skipped),
        )
    }

    pub fn seconds_to_frame_absolute(&self, frame_number: u64) -> f32 {
        self.seconds_between_frames(self.game_time.started_at_frame_number, frame_number)
    }

    pub fn seconds_between_frames(&self, lhs: u64, rhs: u64) -> f32 {
        (lhs as f32 - rhs as f32).abs() * self.engine_time.fixed_seconds()
    }
}
