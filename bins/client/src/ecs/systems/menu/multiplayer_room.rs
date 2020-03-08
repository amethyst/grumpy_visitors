use gv_client_shared::ecs::resources::ConnectionStatus;

use super::*;

const DISCONNECTED: &str = "MP_DISCONNECTED";

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

pub struct MultiplayerRoomMenuScreen;

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
            UI_MP_ROOM_PLAYER2_CONTAINER,
            UI_MP_ROOM_PLAYER2_BG,
            UI_MP_ROOM_PLAYER2_NUMBER,
            UI_MP_ROOM_PLAYER2_NICKNAME,
            // UI_MP_ROOM_PLAYER2_KICK,
            UI_MP_ROOM_PLAYER3_CONTAINER,
            UI_MP_ROOM_PLAYER3_BG,
            UI_MP_ROOM_PLAYER3_NUMBER,
            UI_MP_ROOM_PLAYER3_NICKNAME,
            // UI_MP_ROOM_PLAYER3_KICK,
            UI_MP_ROOM_PLAYER4_CONTAINER,
            UI_MP_ROOM_PLAYER4_BG,
            UI_MP_ROOM_PLAYER4_NUMBER,
            UI_MP_ROOM_PLAYER4_NICKNAME,
            // UI_MP_ROOM_PLAYER4_KICK,
        ]
    }

    fn process_events(
        &mut self,
        system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        modal_window_id: Option<&str>,
    ) -> StateUpdate {
        let state_update = if let ConnectionStatus::ConnectionFailed(ref error) =
            system_data.multiplayer_room_state.connection_status
        {
            Some(StateUpdate::ShowModalWindow {
                id: DISCONNECTED.to_owned(),
                title: error
                    .as_ref()
                    .map(|error| format!("Disconnected: {:?}", error))
                    .unwrap_or_else(|| "Disconnected".to_owned()),
                show_confirmation: true,
            })
        } else {
            None
        };
        if let Some(state_update) = state_update {
            system_data.multiplayer_room_state.connection_status = ConnectionStatus::NotConnected;
            return state_update;
        }

        match (button_pressed, modal_window_id) {
            (Some(UI_MP_ROOM_LOBBY_BUTTON), _) => {
                system_data.multiplayer_room_state.reset();
                system_data.multiplayer_game_state.reset();
                system_data.server_command.kill();
                StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu)
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
            let rows = [
                (UI_MP_ROOM_PLAYER1_NUMBER, UI_MP_ROOM_PLAYER1_NICKNAME),
                (UI_MP_ROOM_PLAYER2_NUMBER, UI_MP_ROOM_PLAYER2_NICKNAME),
                (UI_MP_ROOM_PLAYER3_NUMBER, UI_MP_ROOM_PLAYER3_NICKNAME),
                (UI_MP_ROOM_PLAYER4_NUMBER, UI_MP_ROOM_PLAYER4_NICKNAME),
            ];
            for (i, row) in rows.iter().enumerate() {
                {
                    if players.get(i).is_some() {
                        elements_to_show.push(row.0);
                        elements_to_show.push(row.1);
                    } else {
                        elements_to_hide.push(row.0);
                        elements_to_hide.push(row.1);
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
