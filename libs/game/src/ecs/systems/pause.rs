use amethyst::ecs::{ReadExpect, System, WriteExpect};

use ha_core::ecs::resources::{net::MultiplayerGameState, GameTime};

pub struct PauseSystem;

impl<'s> System<'s> for PauseSystem {
    type SystemData = (
        ReadExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, GameTime>,
    );

    fn run(&mut self, (multiplayer_game_state, mut game_time): Self::SystemData) {
        if multiplayer_game_state.waiting_network || multiplayer_game_state.waiting_for_players {
            game_time.frames_skipped += 1;
            log::info!("Skipping a frame (skipped: {})", game_time.frames_skipped);
        }
    }
}
