use amethyst::{
    assets::Handle,
    core::{HiddenPropagate, Time},
    ecs::{Read, ReadExpect, System, WriteStorage},
    renderer::Material,
    ui::UiText,
    window::ScreenDimensions,
};

use crate::{models::common::GameState, utils::ui::UiFinderMut};

pub struct MenuSystem;

impl<'s> System<'s> for MenuSystem {
    type SystemData = (
        Read<'s, Time>,
        UiFinderMut<'s>,
        ReadExpect<'s, GameState>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, HiddenPropagate>,
        WriteStorage<'s, Handle<Material>>,
    );

    fn run(
        &mut self,
        (
            time,
            mut ui_finder,
            game_state,
            screen_dimensions,
            mut ui_texts,
            mut hidden_propagates,
            mut _materials,
        ): Self::SystemData,
    ) {
        update_container_transform(&mut ui_finder, &screen_dimensions);

        let (ui_loading, ui_background) = if let (Some(ui_loading), Some(ui_background)) = (
            ui_finder.find("ui_loading_label"),
            ui_finder.find("ui_background_container"),
        ) {
            (ui_loading, ui_background)
        } else {
            return;
        };

        if let GameState::Loading = *game_state {
            let dots_count = (time.absolute_real_time_seconds() as usize + 2) % 3 + 1;
            let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
            let ui_loading_text = ui_texts.get_mut(ui_loading).unwrap();
            ui_loading_text.text = "Loading".to_owned() + &dots;
        } else {
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
}

fn update_container_transform(ui_finder: &mut UiFinderMut, screen_dimensions: &ScreenDimensions) {
    if let Some((_, ui_background_transform)) =
        ui_finder.find_with_mut_transform("ui_background_container")
    {
        ui_background_transform.width = screen_dimensions.width();
        ui_background_transform.height = screen_dimensions.height();
    }
}
