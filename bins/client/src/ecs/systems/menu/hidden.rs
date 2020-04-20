use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;
use crate::{ecs::resources::UiNetworkCommand, utils::ui::disconnect_reason_title};

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

        let disconnected_modal_window_is_shown =
            modal_window_id.map_or(false, |modal_window_id| modal_window_id == DISCONNECTED);
        if !disconnected_modal_window_is_shown {
            match system_data.multiplayer_room_state.connection_status {
                ConnectionStatus::ConnectionFailed(ref error) => {
                    return StateUpdate::ShowModalWindow {
                        id: DISCONNECTED.to_owned(),
                        title: error
                            .as_ref()
                            .map(|error| format!("Disconnected: {:?}", error))
                            .unwrap_or_else(|| "Disconnected".to_owned()),
                        show_confirmation: true,
                    }
                }
                ConnectionStatus::Disconnected(disconnect_reason) => {
                    return StateUpdate::ShowModalWindow {
                        id: DISCONNECTED.to_owned(),
                        title: disconnect_reason_title(disconnect_reason),
                        show_confirmation: true,
                    }
                }
                _ => {}
            }
        }

        if let (Some(UI_MODAL_CONFIRM_BUTTON), Some(DISCONNECTED)) =
            (button_pressed, modal_window_id)
        {
            // TODO (refactor): mutating game_level_state from here is not cool.
            system_data.game_level_state.is_over = true;

            system_data.ui_network_command.command = Some(UiNetworkCommand::Reset);
            StateUpdate::GameMenuUpdate {
                game_engine_state: Some(GameEngineState::Menu),
                menu_screen: Some(GameMenuScreen::LobbyMenu),
            }
        } else {
            StateUpdate::None
        }
    }
}
