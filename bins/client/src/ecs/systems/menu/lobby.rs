use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use super::*;

pub struct LobbyMenuScreen;

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
    ) -> StateUpdate {
        match button_pressed {
            Some(UI_MAIN_MENU_BUTTON) => StateUpdate::new_menu_screen(GameMenuScreen::MainMenu),
            Some(b @ UI_LOBBY_HOST_BUTTON) | Some(b @ UI_LOBBY_JOIN_BUTTON) => {
                let is_host = b == UI_LOBBY_HOST_BUTTON;

                let address_field = if is_host {
                    UI_LOBBY_HOST_IP_EDITABLE
                } else {
                    UI_LOBBY_JOIN_IP_EDITABLE
                };
                let addr = system_data.ui_finder
                    .find(address_field)
                    .and_then(|entity| system_data.ui_texts.get(entity))
                    .map(|ui_text| ui_text.text.clone())
                    .unwrap();
                let nickname = system_data.ui_finder
                    .find(UI_LOBBY_NICKNAME_EDITABLE)
                    .and_then(|entity| system_data.ui_texts.get(entity))
                    .map(|ui_text| ui_text.text.clone())
                    .unwrap();

                // TODO: error validations.
                let server_addr = addr.parse().expect("Expected a valid address");
                if is_host {
                    let mut host_client_addr = system_data.laminar_socket
                        .get_mut()
                        .expect("Expected a LaminarSocket")
                        .local_addr()
                        .expect("Expected a local address for a Laminar socket");
                    match &mut host_client_addr {
                        SocketAddr::V4(addr) => addr.set_ip(Ipv4Addr::new(127, 0, 0, 1)),
                        SocketAddr::V6(addr) => addr.set_ip(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                    };
                    system_data.server_command
                        .start(server_addr, host_client_addr)
                        .expect("Expected to start a server");
                }
                system_data.multiplayer_room_state.nickname = nickname;
                system_data.multiplayer_room_state.is_active = true;
                system_data.multiplayer_room_state.server_addr = server_addr;
                system_data.multiplayer_room_state.is_host = is_host;

                StateUpdate::new_menu_screen(GameMenuScreen::MultiplayerRoomMenu)
            }
            _ => StateUpdate::none(),
        }
    }
}
