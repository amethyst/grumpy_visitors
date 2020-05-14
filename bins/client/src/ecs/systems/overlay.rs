use amethyst::{
    core::HiddenPropagate,
    ecs::{ReadExpect, System, WriteStorage},
    ui::UiText,
};

use gv_core::ecs::resources::net::MultiplayerGameState;

use crate::ecs::system_data::ui::UiFinderMut;

pub struct OverlaySystem;

const UI_WAITING_FOR_PLAYERS_BORDER_CONTAINER: &str = "ui_waiting_for_players_border_container";
const UI_WAITING_FOR_PLAYERS_ROW_1: &str = "ui_waiting_for_players_row_1";
const UI_WAITING_FOR_PLAYERS_ROW_2: &str = "ui_waiting_for_players_row_2";
const UI_WAITING_FOR_PLAYERS_ROW_3: &str = "ui_waiting_for_players_row_3";
const UI_WAITING_FOR_PLAYERS_ROW_4: &str = "ui_waiting_for_players_row_4";
const UI_WAITING_FOR_NETWORK_BORDER_CONTAINER: &str = "ui_waiting_for_network_border_container";

impl<'s> System<'s> for OverlaySystem {
    type SystemData = (
        UiFinderMut<'s>,
        ReadExpect<'s, MultiplayerGameState>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, HiddenPropagate>,
    );

    fn run(
        &mut self,
        (mut ui_finder, multiplayer_game_state, mut ui_texts, mut hidden_propagates): Self::SystemData,
    ) {
        if !multiplayer_game_state.is_playing {
            return;
        }

        if multiplayer_game_state.waiting_network {
            hidden_propagates.remove(
                ui_finder
                    .find(UI_WAITING_FOR_NETWORK_BORDER_CONTAINER)
                    .expect("Expected ui_waiting_for_network_border_container ui element"),
            );
        } else {
            hidden_propagates
                .insert(
                    ui_finder
                        .find(UI_WAITING_FOR_NETWORK_BORDER_CONTAINER)
                        .expect("Expected ui_waiting_for_network_border_container ui element"),
                    HiddenPropagate::new(),
                )
                .expect("Expected to insert HiddenPropagate");
        }

        if !multiplayer_game_state.waiting_network && multiplayer_game_state.waiting_for_players {
            let lagging_players = multiplayer_game_state.lagging_players.clone();
            hidden_propagates.remove(
                ui_finder
                    .find(UI_WAITING_FOR_PLAYERS_BORDER_CONTAINER)
                    .expect("Expected ui_waiting_for_players_border_container ui element"),
            );

            let mut update_name = |ui_id: &'static str, lagging_player_index: usize| {
                let player_text_entity = ui_finder
                    .find(ui_id)
                    .unwrap_or_else(|| panic!("Expected {} ui element", ui_id));

                if lagging_players.len() > lagging_player_index {
                    hidden_propagates.remove(player_text_entity);
                    let player_name_text = ui_finder
                        .get_ui_text_mut(&mut ui_texts, ui_id)
                        .unwrap_or_else(|| panic!("Expected {} ui element", ui_id));
                    let player_name = multiplayer_game_state
                        .find_player_by_connection_id(
                            multiplayer_game_state.lagging_players[lagging_player_index],
                        )
                        .expect("Expected to find a lagging player")
                        .nickname
                        .clone();
                    *player_name_text = player_name;
                } else {
                    hidden_propagates
                        .insert(player_text_entity, HiddenPropagate::new())
                        .expect("Expected to insert HiddenPropagate");
                }
            };

            update_name(UI_WAITING_FOR_PLAYERS_ROW_1, 0);
            update_name(UI_WAITING_FOR_PLAYERS_ROW_2, 1);
            update_name(UI_WAITING_FOR_PLAYERS_ROW_3, 2);
            update_name(UI_WAITING_FOR_PLAYERS_ROW_4, 3);
        } else {
            hidden_propagates
                .insert(
                    ui_finder
                        .find(UI_WAITING_FOR_PLAYERS_BORDER_CONTAINER)
                        .expect("Expected ui_waiting_for_players_border_container ui element"),
                    HiddenPropagate::new(),
                )
                .expect("Expected to insert HiddenPropagate");
        }
    }
}
