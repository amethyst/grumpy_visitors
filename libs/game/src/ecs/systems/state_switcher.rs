use amethyst::{
    ecs::{ReadExpect, System, Write},
    prelude::{GameData, StateEvent, Trans, TransEvent},
    shrev::EventChannel,
};

use gv_core::ecs::resources::{GameEngineState, NewGameEngineState};

use crate::states::*;

pub struct StateSwitcherSystem;

impl<'s> System<'s> for StateSwitcherSystem {
    type SystemData = (
        ReadExpect<'s, NewGameEngineState>,
        ReadExpect<'s, GameEngineState>,
        Write<'s, EventChannel<TransEvent<GameData<'static, 'static>, StateEvent>>>,
    );

    fn run(
        &mut self,
        (new_game_engine_state, game_engine_state, mut trans_events): Self::SystemData,
    ) {
        let new_game_engine_state = *new_game_engine_state;
        if *game_engine_state != new_game_engine_state.0 {
            let trans = Box::new(move || match new_game_engine_state.0 {
                GameEngineState::Loading => unreachable!(),
                GameEngineState::Menu => Trans::Switch(Box::new(MenuState)),
                GameEngineState::Playing => Trans::Switch(Box::new(PlayingState)),
                GameEngineState::Quit => Trans::Quit,
            });
            trans_events.single_write(trans);
        }
    }
}
