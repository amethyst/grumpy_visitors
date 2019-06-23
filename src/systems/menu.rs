use amethyst::{
    assets::Handle,
    core::{HiddenPropagate, Time},
    ecs::{Entities, Read, ReadExpect, ReadStorage, System, WriteStorage},
    renderer::Material,
    ui::{UiFinder, UiText},
    utils::tag::{Tag, TagFinder},
};

use crate::{models::common::GameState, tags::UiBackground};

pub struct MenuSystem;

impl<'s> System<'s> for MenuSystem {
    type SystemData = (
        Read<'s, Time>,
        UiFinder<'s>,
        ReadExpect<'s, GameState>,
        Entities<'s>,
        ReadStorage<'s, Tag<UiBackground>>,
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
            entities,
            ui_background_tags,
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
            let tag_finder = TagFinder {
                entities,
                tags: ui_background_tags,
            };
            let ui_background = tag_finder.find().unwrap();
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
