use super::*;

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
    ) -> StateUpdate {
        match button_pressed {
            Some(UI_MP_ROOM_LOBBY_BUTTON) => {
                StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu)
            }
            Some(UI_MP_ROOM_START_BUTTON) => {
                system_data.multiplayer_room_state.has_started = true;
                StateUpdate::none()
            }
            _ => {
                Self::update_players(system_data);
                StateUpdate::none()
            }
        }
    }
}

impl MultiplayerRoomMenuScreen {
    fn update_players(system_data: &mut MenuSystemData) {
        if let Some(players) = system_data.multiplayer_game_state.read_updated_players() {
            let rows = [
                (UI_MP_ROOM_PLAYER1_NUMBER, UI_MP_ROOM_PLAYER1_NICKNAME),
                (UI_MP_ROOM_PLAYER2_NUMBER, UI_MP_ROOM_PLAYER2_NICKNAME),
                (UI_MP_ROOM_PLAYER3_NUMBER, UI_MP_ROOM_PLAYER3_NICKNAME),
                (UI_MP_ROOM_PLAYER4_NUMBER, UI_MP_ROOM_PLAYER4_NICKNAME),
            ];
            for (i, row) in rows.iter().enumerate() {
                {
                    let (ui_number_entity, ui_number_transform) = system_data
                        .ui_finder
                        .find_with_mut_transform(row.0)
                        .unwrap_or_else(|| {
                            panic!("Expected a player number UiTransform for row {}", i)
                        });
                    let ui_number_text = system_data
                        .ui_texts
                        .get_mut(ui_number_entity)
                        .unwrap_or_else(|| panic!("Expected a player number UiText for row {}", i));
                    if players.get(i).is_some() {
                        system_data.hidden_propagates.remove(ui_number_entity);
                        ui_number_transform.local_z = 1.0;
                        ui_number_text.color[3] = 1.0;
                    } else {
                        system_data
                            .hidden_propagates
                            .insert(ui_number_entity, HiddenPropagate)
                            .expect("Expected to insert Hidden component");
                        ui_number_transform.local_z = 0.5;
                        ui_number_text.color[3] = 0.0;
                    }
                }

                {
                    let (ui_text_entity, ui_text_transform) = system_data
                        .ui_finder
                        .find_with_mut_transform(row.1)
                        .unwrap_or_else(|| {
                            panic!("Expected a player nickname UiTransform for row {}", i)
                        });
                    let ui_nickname_text = system_data
                        .ui_texts
                        .get_mut(ui_text_entity)
                        .unwrap_or_else(|| panic!("Expected a player number UiText for row {}", i));

                    if let Some(player) = players.get(i) {
                        system_data.hidden_propagates.remove(ui_text_entity);
                        ui_text_transform.local_z = 1.0;
                        ui_nickname_text.color[3] = 1.0;
                        ui_nickname_text.text = player.nickname.clone();
                    } else {
                        system_data
                            .hidden_propagates
                            .insert(ui_text_entity, HiddenPropagate)
                            .expect("Expected to insert Hidden component");
                        ui_text_transform.local_z = 0.5;
                        ui_nickname_text.color[3] = 0.0;
                    }
                }
            }
        }
    }
}
