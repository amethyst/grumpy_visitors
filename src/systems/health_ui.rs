use amethyst::{
    ecs::{Join, ReadExpect, ReadStorage, System, WriteStorage},
    window::ScreenDimensions,
};

use crate::{
    components::{HealthUiGraphics, Player},
    data_resources::HEALTH_UI_SCREEN_PADDING,
    Vector2,
};

pub struct HealthUiSystem;

impl<'a> System<'a> for HealthUiSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, HealthUiGraphics>,
    );

    fn run(&mut self, (screen_dimensions, players, mut health_uis): Self::SystemData) {
        let half_screen_width = screen_dimensions.width() / 2.0;
        let half_screen_height = screen_dimensions.height() / 2.0;

        for (player, health_ui) in (&players, &mut health_uis).join() {
            health_ui.health = player.health / 100.0;
            health_ui.screen_position = Vector2::new(
                -half_screen_width + HEALTH_UI_SCREEN_PADDING,
                -half_screen_height + HEALTH_UI_SCREEN_PADDING,
            );
        }
    }
}
