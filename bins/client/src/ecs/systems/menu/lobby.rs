use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;
use crate::{ecs::resources::UiNetworkCommand, utils::ui::disconnect_reason_title};

pub struct LobbyMenuScreen;

const INVALID_IP_ADDRESS: &str = "LOBBY_INVALID_IP_ADDRESS";
const SERVER_START_FAILED: &str = "LOBBY_SERVER_START_FAILED";
const CONNECTING_PROGRESS: &str = "LOBBY_CONNECTING_PROGRESS";
const CONNECTING_FAILED: &str = "LOBBY_CONNECTING_FAILED";

impl MenuScreen for LobbyMenuScreen {
    fn elements_to_show(&self, _system_data: &MenuSystemData) -> Vec<MenuElement> {
        vec![
            UI_LOBBY_NICKNAME_LABEL,
            UI_LOBBY_NICKNAME_FIELD,
            UI_LOBBY_NICKNAME_EDITABLE,
            UI_LOBBY_HOST_IP_FIELD,
            UI_LOBBY_HOST_IP_EDITABLE,
            UI_LOBBY_HOST_BUTTON,
            UI_LOBBY_JOIN_IP_FIELD,
            UI_LOBBY_JOIN_IP_EDITABLE,
            UI_LOBBY_JOIN_BUTTON,
            UI_MAIN_MENU_BUTTON,
        ]
    }

    fn update(
        &mut self,
        system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        modal_window_id: Option<&str>,
    ) -> StateUpdate {
        match (button_pressed, modal_window_id) {
            (Some(UI_MAIN_MENU_BUTTON), _) => {
                StateUpdate::new_menu_screen(GameMenuScreen::MainMenu)
            }
            (Some(b @ (UI_LOBBY_HOST_BUTTON | UI_LOBBY_JOIN_BUTTON)), _) => {
                let is_host = b == UI_LOBBY_HOST_BUTTON;

                let address_field = if is_host {
                    UI_LOBBY_HOST_IP_EDITABLE
                } else {
                    UI_LOBBY_JOIN_IP_EDITABLE
                };
                let addr = system_data
                    .ui_finder
                    .get_ui_text(&system_data.ui_texts, address_field)
                    .unwrap();
                let nickname = system_data
                    .ui_finder
                    .get_ui_text(&system_data.ui_texts, UI_LOBBY_NICKNAME_EDITABLE)
                    .cloned()
                    .unwrap();

                let server_addr = addr.parse();
                if server_addr.is_err() {
                    return StateUpdate::ShowModalWindow {
                        id: INVALID_IP_ADDRESS.to_owned(),
                        title: "Server IP address has invalid format".to_owned(),
                        show_confirmation: true,
                    };
                }
                let server_addr = server_addr.unwrap();

                log::info!("Joining {}...", server_addr);
                if is_host {
                    system_data.ui_network_command.command = Some(UiNetworkCommand::Host {
                        nickname,
                        server_addr,
                    });
                } else {
                    system_data.ui_network_command.command = Some(UiNetworkCommand::Connect {
                        nickname,
                        server_addr,
                    });
                }
                StateUpdate::ShowModalWindow {
                    id: CONNECTING_PROGRESS.to_owned(),
                    title: "Connecting...".to_owned(),
                    show_confirmation: false,
                }
            }
            (Some(UI_MODAL_CONFIRM_BUTTON), Some(CONNECTING_FAILED)) => {
                system_data.ui_network_command.command = Some(UiNetworkCommand::Reset);
                StateUpdate::None
            }
            (Some(UI_MODAL_CONFIRM_BUTTON), Some(SERVER_START_FAILED)) => {
                system_data.ui_network_command.command = Some(UiNetworkCommand::Reset);
                StateUpdate::None
            }
            (None, modal_window_id) => {
                match &system_data.multiplayer_room_state.connection_status {
                    ConnectionStatus::NotConnected => StateUpdate::None,
                    ConnectionStatus::Connecting(_) => StateUpdate::None,
                    ConnectionStatus::Disconnecting => StateUpdate::None,
                    ConnectionStatus::Connected(_) => {
                        StateUpdate::new_menu_screen(GameMenuScreen::MultiplayerRoomMenu)
                    }
                    ConnectionStatus::ConnectionFailed(error) => {
                        if is_failed_modal_window(modal_window_id) {
                            StateUpdate::None
                        } else {
                            StateUpdate::ShowModalWindow {
                                id: CONNECTING_FAILED.to_owned(),
                                title: error
                                    .as_ref()
                                    .map(|error| format!("Disconnected: {:?}", error))
                                    .unwrap_or_else(|| "Disconnected".to_owned()),
                                show_confirmation: true,
                            }
                        }
                    }
                    ConnectionStatus::Disconnected(disconnect_reason) => {
                        if is_failed_modal_window(modal_window_id) {
                            StateUpdate::None
                        } else {
                            StateUpdate::ShowModalWindow {
                                id: CONNECTING_FAILED.to_owned(),
                                title: disconnect_reason_title(*disconnect_reason),
                                show_confirmation: true,
                            }
                        }
                    }
                    ConnectionStatus::ServerStartFailed => {
                        if is_failed_modal_window(modal_window_id) {
                            StateUpdate::None
                        } else {
                            StateUpdate::ShowModalWindow {
                                id: SERVER_START_FAILED.to_owned(),
                                title: "Couldn't start the server, make sure that gv_server is in the same working directory".to_owned(),
                                show_confirmation: true,
                            }
                        }
                    }
                }
            }
            _ => StateUpdate::None,
        }
    }
}

fn is_failed_modal_window(modal_window_id: Option<&str>) -> bool {
    modal_window_id.map_or(true, |modal_window_id| {
        modal_window_id == CONNECTING_FAILED || modal_window_id == SERVER_START_FAILED
    })
}
