use amethyst::{
    ecs::{ReadExpect, System},
    window::ScreenDimensions,
};
use amethyst_imgui::imgui::{self, im_str};

use gv_core::ecs::resources::{
    net::{MultiplayerGameState, PlayersNetStatus},
    GameEngineState,
};

use crate::ecs::resources::DisplayDebugInfoSettings;

pub struct ImguiNetworkDebugInfoSystem;

impl<'s> System<'s> for ImguiNetworkDebugInfoSystem {
    type SystemData = (
        ReadExpect<'s, GameEngineState>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, MultiplayerGameState>,
        ReadExpect<'s, DisplayDebugInfoSettings>,
        ReadExpect<'s, PlayersNetStatus>,
    );

    fn run(
        &mut self,
        (
            game_engine_state,
            screen_dimensions,
            multiplayer_game_state,
            display_debug_info_settings,
            players_net_status,
        ): Self::SystemData,
    ) {
        if !game_engine_state.is_playing() {
            return;
        }

        amethyst_imgui::with(|ui| {
            if display_debug_info_settings.display_network_debug_info {
                imgui::Window::new(im_str!("Network Debug Info"))
                    .title_bar(false)
                    .movable(false)
                    .resizable(false)
                    .save_settings(false)
                    .collapsible(false)
                    .focused(false)
                    .focus_on_appearing(false)
                    .no_nav()
                    .position(
                        [
                            screen_dimensions.width() / screen_dimensions.hidpi_factor() as f32,
                            0.0,
                        ],
                        imgui::Condition::Always,
                    )
                    .position_pivot([1.1, -0.1])
                    .size([250.0, 150.0], imgui::Condition::Always)
                    .bg_alpha(0.7)
                    .build(ui, || {
                        if !multiplayer_game_state.is_playing {
                            ui.text("It's not a multiplayer game");
                            return;
                        }

                        ui.columns(4, im_str!("Network Debug Info"), false);
                        ui.text("Name");
                        ui.next_column();
                        ui.text("Diff");
                        ui.next_column();
                        ui.text("Behind");
                        ui.next_column();
                        ui.text("Latency");
                        for player in &multiplayer_game_state.players {
                            let player_net_status = players_net_status
                                .players
                                .iter()
                                .cloned()
                                .find(|player_net_status| {
                                    player_net_status.connection_id == player.connection_id
                                })
                                .unwrap_or_default();
                            let frame_diff = players_net_status.frame_received as i32
                                - player_net_status.frame_number as i32;

                            ui.next_column();
                            ui.text(player.nickname.to_string());
                            ui.next_column();
                            ui.text(frame_diff.to_string());
                            ui.next_column();
                            ui.text(player_net_status.average_lagging_behind.to_string());
                            ui.next_column();
                            ui.text(player_net_status.latency_ms.to_string());
                        }
                    });
            }
        });
    }
}
