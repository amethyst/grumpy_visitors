use amethyst::{
    ecs::{prelude::World, ReadExpect},
    shred::{ResourceId, SystemData},
};

use crate::ecs::resources::{net::MultiplayerGameState, GameEngineState};

#[derive(SystemData)]
pub struct GameStateHelper<'s> {
    game_engine_state: ReadExpect<'s, GameEngineState>,
    multiplayer_game_state: ReadExpect<'s, MultiplayerGameState>,
}

impl<'s> GameStateHelper<'s> {
    pub fn is_running(&self) -> bool {
        let is_playing_multiplayer = self.multiplayer_game_state.is_playing;
        let multiplayer_is_unpaused = !is_playing_multiplayer
            || (!self.multiplayer_game_state.waiting_network
                && !self.multiplayer_game_state.waiting_for_players);

        *self.game_engine_state == GameEngineState::Playing && multiplayer_is_unpaused
    }

    pub fn multiplayer_is_running(&self) -> bool {
        *self.game_engine_state == GameEngineState::Playing && self.multiplayer_is_unpaused()
    }

    pub fn multiplayer_is_unpaused(&self) -> bool {
        self.multiplayer_game_state.is_playing
            && !self.multiplayer_game_state.waiting_network
            && !self.multiplayer_game_state.waiting_for_players
    }
}
