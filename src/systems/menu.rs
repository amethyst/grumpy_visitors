use amethyst::{
    assets::Handle,
    core::{HiddenPropagate, Time},
    ecs::{Read, ReadExpect, System, WriteStorage},
    renderer::Material,
    ui::{UiFinder, UiText},
};

use crate::models::common::GameState;

pub struct MenuSystem;

impl<'s> System<'s> for MenuSystem {
    type SystemData = (
        Read<'s, Time>,
        UiFinder<'s>,
        ReadExpect<'s, GameState>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, HiddenPropagate>,
        WriteStorage<'s, Handle<Material>>,
    );

    fn run(
        &mut self,
        (
            time,
            ui_finder,
            game_state,
            mut ui_texts,
            mut hidden_propagates,
            mut _materials,
        ): Self::SystemData,
    ) {
        let ui_loading = ui_finder.find("ui_loading").unwrap();

        if let GameState::Loading = *game_state {
            let dots_count = (time.absolute_real_time_seconds() as usize + 2) % 3 + 1;
            let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
            let ui_loading_text = ui_texts.get_mut(ui_loading).unwrap();
            ui_loading_text.text = "Loading".to_owned() + &dots;
        } else {
            let ui_background = ui_finder.find("ui_background").unwrap();
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
