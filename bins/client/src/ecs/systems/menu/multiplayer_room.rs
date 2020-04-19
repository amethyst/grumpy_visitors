use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;
use crate::{ecs::resources::UiNetworkCommand, utils::ui::disconnect_reason_title};

const DISCONNECTED: &str = "MP_DISCONNECTED";
const DISCONNECTING: &str = "MP_DISCONNECTING";

lazy_static! {
    static ref MP_ROOM_MENU_ELEMENTS_HOST: &'static [&'static str] = &[
        UI_MP_ROOM_START_BUTTON,
        UI_MP_ROOM_LOBBY_BUTTON,
        UI_MP_ROOM_PLAYER1_CONTAINER,
        UI_MP_ROOM_PLAYER1_BG,
        UI_MP_ROOM_PLAYER2_CONTAINER,
        UI_MP_ROOM_PLAYER2_BG,
        UI_MP_ROOM_PLAYER3_CONTAINER,
        UI_MP_ROOM_PLAYER3_BG,
        UI_MP_ROOM_PLAYER4_CONTAINER,
        UI_MP_ROOM_PLAYER4_BG,
    ];
    static ref MP_ROOM_MENU_ELEMENTS_JOIN: &'static [&'static str] = &[
        UI_MP_ROOM_LOBBY_BUTTON,
        UI_MP_ROOM_PLAYER1_CONTAINER,
        UI_MP_ROOM_PLAYER1_BG,
        UI_MP_ROOM_PLAYER2_CONTAINER,
        UI_MP_ROOM_PLAYER2_BG,
        UI_MP_ROOM_PLAYER3_CONTAINER,
        UI_MP_ROOM_PLAYER3_BG,
        UI_MP_ROOM_PLAYER4_CONTAINER,
        UI_MP_ROOM_PLAYER4_BG,
    ];
}

pub struct MultiplayerRoomMenuScreen {
    initiated_disconnecting: bool,
}

impl MultiplayerRoomMenuScreen {
    pub fn new() -> Self {
        Self {
            initiated_disconnecting: false,
        }
    }
}

impl MenuScreen for MultiplayerRoomMenuScreen {
    fn elements_to_show(&self, system_data: &MenuSystemData) -> Vec<MenuElement> {
        if system_data.multiplayer_room_state.is_host {
            MP_ROOM_MENU_ELEMENTS_HOST.to_vec()
        } else {
            MP_ROOM_MENU_ELEMENTS_JOIN.to_vec()
        }
    }

    fn elements_to_hide(&self, _system_data: &MenuSystemData) -> Vec<&'static str> {
        vec![
            UI_MP_ROOM_START_BUTTON,
            UI_MP_ROOM_LOBBY_BUTTON,
            UI_MP_ROOM_PLAYER1_CONTAINER,
            UI_MP_ROOM_PLAYER1_BG,
            UI_MP_ROOM_PLAYER1_NUMBER,
            UI_MP_ROOM_PLAYER1_NICKNAME,
            UI_MP_ROOM_PLAYER1_KICK,
            UI_MP_ROOM_PLAYER2_CONTAINER,
            UI_MP_ROOM_PLAYER2_BG,
            UI_MP_ROOM_PLAYER2_NUMBER,
            UI_MP_ROOM_PLAYER2_NICKNAME,
            UI_MP_ROOM_PLAYER2_KICK,
            UI_MP_ROOM_PLAYER3_CONTAINER,
            UI_MP_ROOM_PLAYER3_BG,
            UI_MP_ROOM_PLAYER3_NUMBER,
            UI_MP_ROOM_PLAYER3_NICKNAME,
            UI_MP_ROOM_PLAYER3_KICK,
            UI_MP_ROOM_PLAYER4_CONTAINER,
            UI_MP_ROOM_PLAYER4_BG,
            UI_MP_ROOM_PLAYER4_NUMBER,
            UI_MP_ROOM_PLAYER4_NICKNAME,
            UI_MP_ROOM_PLAYER4_KICK,
        ]
    }

    fn update(
        &mut self,
        system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        modal_window_id: Option<&str>,
    ) -> StateUpdate {
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
                if self.initiated_disconnecting {
                    self.initiated_disconnecting = false;
                    system_data.multiplayer_room_state.reset();
                    system_data.multiplayer_game_state.reset();
                    Some(StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu))
                } else {
                    Some(StateUpdate::ShowModalWindow {
                        id: DISCONNECTED.to_owned(),
                        title: disconnect_reason_title(disconnect_reason),
                        show_confirmation: true,
                    })
                }
            }
            _ => None,
        };
        if let Some(state_update) = state_update {
            system_data.multiplayer_room_state.connection_status = ConnectionStatus::NotConnected;
            return state_update;
        }

        match (button_pressed, modal_window_id) {
            (Some(UI_MP_ROOM_PLAYER1_KICK), _) => {
                system_data.ui_network_command.command =
                    Some(UiNetworkCommand::Kick { player_number: 0 });
                StateUpdate::None
            }
            (Some(UI_MP_ROOM_PLAYER2_KICK), _) => {
                system_data.ui_network_command.command =
                    Some(UiNetworkCommand::Kick { player_number: 1 });
                StateUpdate::None
            }
            (Some(UI_MP_ROOM_PLAYER3_KICK), _) => {
                system_data.ui_network_command.command =
                    Some(UiNetworkCommand::Kick { player_number: 2 });
                StateUpdate::None
            }
            (Some(UI_MP_ROOM_PLAYER4_KICK), _) => {
                system_data.ui_network_command.command =
                    Some(UiNetworkCommand::Kick { player_number: 3 });
                StateUpdate::None
            }
            (Some(UI_MP_ROOM_LOBBY_BUTTON), _) => {
                self.initiated_disconnecting = true;
                system_data.multiplayer_room_state.pending_disconnecting = true;
                system_data.multiplayer_room_state.connection_status =
                    ConnectionStatus::Disconnecting;

                if system_data.multiplayer_room_state.is_host {
                    StateUpdate::ShowModalWindow {
                        id: DISCONNECTING.to_owned(),
                        title: "Shutting down the server...".to_owned(),
                        show_confirmation: false,
                    }
                } else {
                    StateUpdate::ShowModalWindow {
                        id: DISCONNECTING.to_owned(),
                        title: "Disconnecting...".to_owned(),
                        show_confirmation: false,
                    }
                }
            }
            (Some(UI_MP_ROOM_START_BUTTON), _) => {
                system_data.multiplayer_room_state.has_started = true;
                StateUpdate::None
            }
            (Some(UI_MODAL_CONFIRM_BUTTON), Some(DISCONNECTED)) => {
                system_data.multiplayer_room_state.reset();
                system_data.multiplayer_game_state.reset();
                StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu)
            }
            _ => Self::update_players(system_data),
        }
    }
}

impl MultiplayerRoomMenuScreen {
    fn update_players(system_data: &mut MenuSystemData) -> StateUpdate {
        let mut elements_to_hide = Vec::new();
        let mut elements_to_show = Vec::new();

        if let Some(players) = system_data.multiplayer_game_state.read_updated_players() {
            #[rustfmt::skip]
            let rows = [
                (UI_MP_ROOM_PLAYER1_NUMBER, UI_MP_ROOM_PLAYER1_NICKNAME, UI_MP_ROOM_PLAYER1_KICK),
                (UI_MP_ROOM_PLAYER2_NUMBER, UI_MP_ROOM_PLAYER2_NICKNAME, UI_MP_ROOM_PLAYER2_KICK),
                (UI_MP_ROOM_PLAYER3_NUMBER, UI_MP_ROOM_PLAYER3_NICKNAME, UI_MP_ROOM_PLAYER3_KICK),
                (UI_MP_ROOM_PLAYER4_NUMBER, UI_MP_ROOM_PLAYER4_NICKNAME, UI_MP_ROOM_PLAYER4_KICK),
            ];
            for (i, row) in rows.iter().enumerate() {
                {
                    if let Some(player) = players.get(i) {
                        let player_nickname_text = system_data
                            .ui_finder
                            .get_ui_text_mut(&mut system_data.ui_texts, row.1)
                            .expect("Expected a player nickname text component");
                        *player_nickname_text = player.nickname.clone();

                        elements_to_show.push(row.0);
                        elements_to_show.push(row.1);
                        if system_data.multiplayer_room_state.is_host && !player.is_host {
                            elements_to_show.push(row.2);
                        }
                    } else {
                        elements_to_hide.push(row.0);
                        elements_to_hide.push(row.1);
                        elements_to_hide.push(row.2);
                    }
                }
            }
        }

        if elements_to_hide.is_empty() && elements_to_show.is_empty() {
            StateUpdate::None
        } else {
            StateUpdate::CustomAnimation {
                elements_to_hide,
                elements_to_show,
            }
        }
    }
}
