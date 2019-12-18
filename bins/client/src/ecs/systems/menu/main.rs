use super::*;

pub struct MainMenuScreen;

impl MenuScreen for MainMenuScreen {
    fn elements_to_show(&self, _system_data: &MenuSystemData) -> Vec<MenuElement> {
        vec![
            UI_SINGLE_PLAYER_BUTTON,
            UI_MULTIPLAYER_BUTTON,
            UI_QUIT_BUTTON,
        ]
    }

    fn process_events(
        &mut self,
        _system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
    ) -> StateUpdate {
        match button_pressed {
            Some(UI_SINGLE_PLAYER_BUTTON) => StateUpdate {
                game_engine_state: Some(GameEngineState::Playing),
                menu_screen: Some(GameMenuScreen::Hidden),
            },
            Some(UI_MULTIPLAYER_BUTTON) => StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu),
            Some(UI_QUIT_BUTTON) => StateUpdate::new_game_engine_state(GameEngineState::Quit),
            _ => StateUpdate::none(),
        }
    }
}
