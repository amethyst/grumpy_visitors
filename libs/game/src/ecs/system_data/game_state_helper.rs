use amethyst::{
    ecs::{prelude::World, ReadExpect},
    shred::{ResourceId, SystemData},
};

use gv_core::ecs::resources::{net::MultiplayerGameState, GameEngineState, NewGameEngineState};

#[derive(SystemData)]
pub struct GameStateHelper<'s> {
    game_engine_state: ReadExpect<'s, GameEngineState>,
    new_game_engine_state: ReadExpect<'s, NewGameEngineState>,
    multiplayer_game_state: ReadExpect<'s, MultiplayerGameState>,
}

impl<'s> GameStateHelper<'s> {
    pub fn is_running(&self) -> bool {
        let is_playing_multiplayer = self.multiplayer_game_state.is_playing;
        let multiplayer_is_unpaused = !is_playing_multiplayer
            || (!self.multiplayer_game_state.waiting_network
                && !self.multiplayer_game_state.waiting_for_players);

        *self.game_engine_state == GameEngineState::Playing
            && self.new_game_engine_state.0 == GameEngineState::Playing
            && multiplayer_is_unpaused
    }

    pub fn is_multiplayer(&self) -> bool {
        self.multiplayer_game_state.is_playing
    }

    #[cfg(feature = "client")]
    pub fn is_authoritative(&self) -> bool {
        !self.is_multiplayer()
    }

    #[cfg(not(feature = "client"))]
    pub fn is_authoritative(&self) -> bool {
        true
    }

    pub fn multiplayer_is_running(&self) -> bool {
        *self.game_engine_state == GameEngineState::Playing
            && self.new_game_engine_state.0 == GameEngineState::Playing
            && self.multiplayer_is_unpaused()
    }

    pub fn multiplayer_is_unpaused(&self) -> bool {
        self.multiplayer_game_state.is_playing
            && !self.multiplayer_game_state.waiting_network
            && !self.multiplayer_game_state.waiting_for_players
            && !self.multiplayer_game_state.is_disconnected
    }
}
