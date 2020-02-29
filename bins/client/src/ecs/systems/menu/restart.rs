use super::*;

pub struct RestartMenuScreen;

impl MenuScreen for RestartMenuScreen {
    fn elements_to_show(&self, _system_data: &MenuSystemData) -> Vec<MenuElement> {
        vec![UI_RESTART_BUTTON, UI_MAIN_MENU_BUTTON]
    }

    fn process_events(
        &mut self,
        _system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        _modal_window_id: Option<&str>,
    ) -> StateUpdate {
        match button_pressed {
            Some(UI_RESTART_BUTTON) => StateUpdate::new_game_engine_state(GameEngineState::Playing),
            Some(UI_MAIN_MENU_BUTTON) => StateUpdate::new_menu_screen(GameMenuScreen::MainMenu),
            _ => StateUpdate::None,
        }
    }
}
