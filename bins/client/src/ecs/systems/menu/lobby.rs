use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;

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

    fn process_events(
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

                if is_host {
                    let mut host_client_addr = system_data
                        .laminar_socket
                        .get_mut()
                        .expect("Expected a LaminarSocket")
                        .local_addr()
                        .expect("Expected a local address for a Laminar socket");
                    match &mut host_client_addr {
                        SocketAddr::V4(addr) => addr.set_ip(Ipv4Addr::new(127, 0, 0, 1)),
                        SocketAddr::V6(addr) => addr.set_ip(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                    };
                    if let Err(err) = system_data
                        .server_command
                        .start(server_addr, host_client_addr)
                    {
                        log::error!("Couldn't start the server: {:?}", err);
                        return StateUpdate::ShowModalWindow {
                            id: SERVER_START_FAILED.to_owned(),
                            title: "Couldn't start the server, make sure that gv_server is in the same working directory".to_owned(),
                            show_confirmation: true,
                        };
                    }
                }
                system_data.multiplayer_room_state.nickname = nickname;
                system_data.multiplayer_room_state.is_active = true;
                system_data.multiplayer_room_state.server_addr = server_addr;
                system_data.multiplayer_room_state.is_host = is_host;

                StateUpdate::ShowModalWindow {
                    id: CONNECTING_PROGRESS.to_owned(),
                    title: "Connecting...".to_owned(),
                    show_confirmation: false,
                }
            }
            (None, _) => {
                let (new_connection_status, state_update) =
                    match &system_data.multiplayer_room_state.connection_status {
                        ConnectionStatus::NotConnected => (None, StateUpdate::None),
                        ConnectionStatus::Connected(_) => (
                            None,
                            StateUpdate::new_menu_screen(GameMenuScreen::MultiplayerRoomMenu),
                        ),
                        ConnectionStatus::ConnectionFailed(error) => (
                            Some(ConnectionStatus::NotConnected),
                            StateUpdate::ShowModalWindow {
                                id: CONNECTING_FAILED.to_owned(),
                                title: error
                                    .as_ref()
                                    .map(|error| format!("Disconnected: {:?}", error))
                                    .unwrap_or_else(|| "Disconnected".to_owned()),
                                show_confirmation: true,
                            },
                        ),
                    };

                if let Some(new_connection_status) = new_connection_status {
                    system_data.multiplayer_room_state.connection_status = new_connection_status;
                }
                state_update
            }
            _ => StateUpdate::None,
        }
    }
}
