use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;
use crate::utils::ui::disconnect_reason_title;

const DISCONNECTED: &str = "MP_GAME_DISCONNECTED";

pub struct HiddenMenuScreen;

impl MenuScreen for HiddenMenuScreen {
    fn elements_to_show(&self, _system_data: &MenuSystemData) -> Vec<MenuElement> {
        vec![]
    }

    fn update(
        &mut self,
        system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        modal_window_id: Option<&str>,
    ) -> StateUpdate {
        if system_data.game_level_state.is_over {
            return StateUpdate::GameMenuUpdate {
                game_engine_state: Some(GameEngineState::Menu),
                menu_screen: Some(GameMenuScreen::RestartMenu),
            };
        }

        let state_update = match system_data.multiplayer_room_state.connection_status {
            ConnectionStatus::ConnectionFailed(ref error) => Some(StateUpdate::ShowModalWindow {
                id: DISCONNECTED.to_owned(),
                title: error
                    .as_ref()
                    .map(|error| format!("Disconnected: {:?}", error))
                    .unwrap_or_else(|| "Disconnected".to_owned()),
                show_confirmation: true,
            }),
            ConnectionStatus::Disconnected(disconnect_reason) => {
                Some(StateUpdate::ShowModalWindow {
                    id: DISCONNECTED.to_owned(),
                    title: disconnect_reason_title(disconnect_reason),
                    show_confirmation: true,
                })
            }
            _ => None,
        };
        if let Some(state_update) = state_update {
            system_data.multiplayer_room_state.connection_status = ConnectionStatus::NotConnected;
            return state_update;
        }

        if let (Some(UI_MODAL_CONFIRM_BUTTON), Some(DISCONNECTED)) =
            (button_pressed, modal_window_id)
        {
            system_data.game_level_state.is_over = true;
            system_data.multiplayer_room_state.reset();
            system_data.multiplayer_game_state.reset();
            StateUpdate::GameMenuUpdate {
                game_engine_state: Some(GameEngineState::Menu),
                menu_screen: Some(GameMenuScreen::LobbyMenu),
            }
        } else {
            StateUpdate::None
        }
    }
}
