use amethyst::{
    core::HiddenPropagate,
    ecs::{ReadExpect, System, WriteStorage},
    ui::{UiImage, UiText},
    window::ScreenDimensions,
};

use std::time::Duration;

use crate::{
    models::common::GameState,
    utils::{
        time::GameTimeService,
        ui::{update_fullscreen_container, UiFinderMut},
    },
};

const MENU_FADE_OUT_DURATION_MS: u64 = 500;

pub struct MenuSystem {
    menu_hidden: bool,
}

impl MenuSystem {
    pub fn new() -> Self {
        Self { menu_hidden: false }
    }
}

impl<'s> System<'s> for MenuSystem {
    type SystemData = (
        GameTimeService<'s>,
        UiFinderMut<'s>,
        ReadExpect<'s, GameState>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, HiddenPropagate>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            mut ui_finder,
            game_state,
            screen_dimensions,
            mut ui_texts,
            mut ui_images,
            mut hidden_propagates,
        ): Self::SystemData,
    ) {
        update_fullscreen_container(
            &mut ui_finder,
            "ui_background_container",
            &screen_dimensions,
        );

        let (ui_loading, ui_background) = if let (Some(ui_loading), Some(ui_background)) = (
            ui_finder.find("ui_loading_label"),
            ui_finder.find("ui_background_container"),
        ) {
            (ui_loading, ui_background)
        } else {
            return;
        };

        match *game_state {
            GameState::Loading => {
                let dots_count =
                    (game_time_service.engine_time().absolute_real_time_seconds() as usize + 2) % 3
                        + 1;
                let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
                let ui_loading_text = ui_texts.get_mut(ui_loading).unwrap();
                ui_loading_text.text = "Loading".to_owned() + &dots;

                self.menu_hidden = false;
            }
            GameState::Playing if !self.menu_hidden => {
                let level_duration = game_time_service.level_duration();
                if level_duration < Duration::from_millis(MENU_FADE_OUT_DURATION_MS) {
                    let alpha = num::Float::max(
                        0.0,
                        1.0 - level_duration.as_millis() as f32 / MENU_FADE_OUT_DURATION_MS as f32,
                    );
                    ui_texts.get_mut(ui_loading).unwrap().color[3] = alpha;
                    if let Some(UiImage::SolidColor(ref mut color)) =
                        ui_images.get_mut(ui_background)
                    {
                        color[3] = alpha;
                    }
                } else {
                    self.menu_hidden = true;

                    if !hidden_propagates.contains(ui_background) {
                        hidden_propagates
                            .insert(ui_background, HiddenPropagate)
                            .unwrap();
                    }

                    if !hidden_propagates.contains(ui_loading) {
                        hidden_propagates
                            .insert(ui_loading, HiddenPropagate)
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}
