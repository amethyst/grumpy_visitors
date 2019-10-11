use amethyst::ecs::{ReadExpect, System, WriteExpect};

use ha_core::ecs::resources::{net::MultiplayerGameState, GameTime};

pub struct PauseSystem;

impl<'s> System<'s> for PauseSystem {
    type SystemData = (
        ReadExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, GameTime>,
    );

    fn run(&mut self, (multiplayer_game_state, mut game_time): Self::SystemData) {
        if multiplayer_game_state.waiting_network {
            game_time.frames_skipped += 1;
            log::info!(
                "Skipping a frame, reason: waiting for network (skipped: {})",
                game_time.frames_skipped
            );
        } else if multiplayer_game_state.waiting_for_players {
            game_time.frames_skipped += 1;
            log::info!(
                "Skipping a frame, reason: waiting for players (skipped: {})",
                game_time.frames_skipped
            );
        }
    }
}
