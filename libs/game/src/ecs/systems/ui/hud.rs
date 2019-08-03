use amethyst::{
    ecs::{Join, ReadExpect, ReadStorage, System, WriteStorage},
    ui::UiText,
    window::ScreenDimensions,
};

use ha_core::math::Vector2;

use crate::{
    ecs::{
        components::{HealthUiGraphics, Player},
        resources::HEALTH_UI_SCREEN_PADDING,
    },
    utils::ui::UiFinderMut,
};

pub struct HealthUiSystem;

impl<'s> System<'s> for HealthUiSystem {
    type SystemData = (
        UiFinderMut<'s>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, HealthUiGraphics>,
        WriteStorage<'s, UiText>,
    );

    fn run(
        &mut self,
        (ui_finder, screen_dimensions, players, mut health_uis, mut ui_texts): Self::SystemData,
    ) {
        let half_screen_width = screen_dimensions.width() / 2.0;
        let half_screen_height = screen_dimensions.height() / 2.0;

        for (player, health_ui) in (&players, &mut health_uis).join() {
            health_ui.health = player.health / 100.0;
            health_ui.screen_position = Vector2::new(
                -half_screen_width + HEALTH_UI_SCREEN_PADDING,
                -half_screen_height + HEALTH_UI_SCREEN_PADDING,
            );

            if let Some(ui_health_label) = ui_finder.find("ui_health_label") {
                ui_texts.get_mut(ui_health_label).unwrap().text =
                    format!("{:.0}/100", num::Float::max(0.0, player.health));
            }
        }
    }
}
