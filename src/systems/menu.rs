use amethyst::{
    core::{HiddenPropagate, ParentHierarchy},
    ecs::{Entity, ReadExpect, System, Write, WriteStorage},
    prelude::{GameData, StateEvent, Trans, TransEvent},
    shrev::{EventChannel, ReaderId},
    ui::{Interactable, UiEvent, UiEventType, UiImage, UiText},
};
use lazy_static::lazy_static;

use std::time::Duration;

use crate::{
    data_resources::{GameEngineState, GameLevelState},
    states::{MenuState, PlayingState},
    utils::{time::GameTimeService, ui::UiFinderMut},
};

const MENU_FADE_OUT_DURATION_MS: u64 = 500;
const UI_BACKGROUND_CONTAINER: &str = "ui_background_container";
const UI_LOADING_LABEL: &str = "ui_loading_label";
const UI_SINGLE_PLAYER_BUTTON: &str = "ui_single_player_button";
const UI_MULTIPLAYER_BUTTON: &str = "ui_multiplayer_button";
const UI_QUIT_BUTTON: &str = "ui_quit_button";
const UI_RESTART_BUTTON: &str = "ui_restart_button";
const UI_MAIN_MENU_BUTTON: &str = "ui_main_menu_button";

lazy_static! {
    static ref MAIN_MENU_ELEMENTS: &'static [&'static str] = &[
        UI_SINGLE_PLAYER_BUTTON,
        UI_MULTIPLAYER_BUTTON,
        UI_QUIT_BUTTON,
    ];
    static ref RESTART_MENU_ELEMENTS: &'static [&'static str] =
        &[UI_RESTART_BUTTON, UI_MAIN_MENU_BUTTON];
}

pub struct MenuSystem {
    elements_to_hide: Vec<&'static str>,
    elements_to_show: Vec<&'static str>,
    mouse_reactive: Vec<&'static str>,
    is_transitioning: bool,
    transition_began_at: Duration,
    event_readers: Option<ReaderId<UiEvent>>,
    menu_state: GameMenuState,
    transition_state: TransitionState,
}

impl MenuSystem {
    pub fn new() -> Self {
        Self {
            elements_to_hide: Vec::new(),
            elements_to_show: Vec::new(),
            mouse_reactive: vec![
                UI_SINGLE_PLAYER_BUTTON,
                UI_QUIT_BUTTON,
                UI_RESTART_BUTTON,
                UI_MAIN_MENU_BUTTON,
            ],
            is_transitioning: false,
            transition_began_at: Duration::new(0, 0),
            event_readers: None,
            menu_state: GameMenuState::Loading,
            transition_state: TransitionState::Still,
        }
    }
}

pub enum GameMenuState {
    Loading,
    MainMenu,
    RestartMenu,
    Hidden,
}

pub enum TransitionState {
    FadeIn,
    FadeOut,
    Still,
}

impl<'s> System<'s> for MenuSystem {
    type SystemData = (
        GameTimeService<'s>,
        UiFinderMut<'s>,
        ReadExpect<'s, ParentHierarchy>,
        ReadExpect<'s, GameEngineState>,
        ReadExpect<'s, GameLevelState>,
        Write<'s, EventChannel<TransEvent<GameData<'static, 'static>, StateEvent>>>,
        Write<'s, EventChannel<UiEvent>>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, Interactable>,
        WriteStorage<'s, HiddenPropagate>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            mut ui_finder,
            hierarchy,
            game_engine_state,
            game_level_state,
            mut trans_events,
            mut ui_events,
            mut ui_texts,
            mut ui_images,
            mut ui_interactables,
            mut hidden_propagates,
        ): Self::SystemData,
    ) {
        let now = game_time_service.engine_time().absolute_real_time();

        let event_readers = self
            .event_readers
            .get_or_insert_with(|| ui_events.register_reader());

        let mut button_pressed = None;
        for event in ui_events.read(event_readers) {
            log::trace!(
                "[SYSTEM] You just interacted with a ui element: {:?}",
                event
            );
            if let UiEventType::Click = event.event_type {
                button_pressed = ui_finder.get_id_by_entity(event.target);
            }
        }

        if let Some(ui_loading) = ui_finder.find(UI_LOADING_LABEL) {
            let dots_count = (now.as_secs() as usize + 2) % 3 + 1;
            let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
            let ui_loading_text = ui_texts.get_mut(ui_loading).unwrap();
            ui_loading_text.text = "Loading".to_owned() + &dots;
        }

        self.run_fade_animation(
            &mut ui_finder,
            &mut ui_texts,
            &mut ui_images,
            &mut ui_interactables,
            &mut hidden_propagates,
            &hierarchy,
            now,
        );

        match (&*game_engine_state, &mut self.menu_state) {
            (GameEngineState::Loading, ref mut menu_state) => {
                **menu_state = GameMenuState::Loading;
            }
            (GameEngineState::Menu, ref mut menu_state @ GameMenuState::Loading) => {
                **menu_state = GameMenuState::MainMenu;
                self.set_fade_animation(now, vec![UI_LOADING_LABEL], MAIN_MENU_ELEMENTS.to_vec());
            }
            (GameEngineState::Menu, ref mut menu_state @ GameMenuState::MainMenu) => {
                match button_pressed.as_ref().map(std::string::String::as_str) {
                    Some(UI_SINGLE_PLAYER_BUTTON) => {
                        **menu_state = GameMenuState::Hidden;
                        trans_events
                            .single_write(Box::new(|| Trans::Switch(Box::new(PlayingState))));
                        self.set_fade_animation(
                            now,
                            with_background(*MAIN_MENU_ELEMENTS),
                            Vec::new(),
                        );
                    }
                    Some(UI_QUIT_BUTTON) => {
                        trans_events.single_write(Box::new(|| Trans::Quit));
                    }
                    _ => {}
                }
            }
            (GameEngineState::Menu, ref mut menu_state @ GameMenuState::RestartMenu) => {
                match button_pressed.as_ref().map(std::string::String::as_str) {
                    Some(UI_RESTART_BUTTON) => {
                        **menu_state = GameMenuState::Hidden;
                        trans_events
                            .single_write(Box::new(|| Trans::Switch(Box::new(PlayingState))));
                        self.set_fade_animation(
                            now,
                            with_background(*RESTART_MENU_ELEMENTS),
                            Vec::new(),
                        );
                    }
                    Some(UI_MAIN_MENU_BUTTON) => {
                        **menu_state = GameMenuState::MainMenu;
                        self.set_fade_animation(
                            now,
                            RESTART_MENU_ELEMENTS.to_vec(),
                            MAIN_MENU_ELEMENTS.to_vec(),
                        );
                    }
                    _ => {}
                }
            }
            (GameEngineState::Playing, ref mut menu_state) if game_level_state.is_over => {
                **menu_state = GameMenuState::RestartMenu;
                self.set_fade_animation(now, Vec::new(), with_background(*RESTART_MENU_ELEMENTS));
                trans_events.single_write(Box::new(|| Trans::Switch(Box::new(MenuState))));
            }
            _ => {}
        }
    }
}

impl MenuSystem {
    fn set_fade_animation(
        &mut self,
        begin_time: Duration,
        elements_to_hide: Vec<&'static str>,
        elements_to_show: Vec<&'static str>,
    ) {
        if let TransitionState::Still = self.transition_state {
        } else {
            panic!("Transition state must be Still before new transition");
        }

        if !elements_to_hide.is_empty() {
            self.transition_state = TransitionState::FadeOut;
        } else if !elements_to_show.is_empty() {
            self.transition_state = TransitionState::FadeIn;
        } else {
            panic!("There's no elements to show or hide");
        }

        self.transition_began_at = begin_time;
        self.elements_to_hide = elements_to_hide;
        self.elements_to_show = elements_to_show;
        self.is_transitioning = true;
    }

    fn run_fade_animation(
        &mut self,
        ui_finder: &mut UiFinderMut<'_>,
        ui_texts: &mut WriteStorage<'_, UiText>,
        ui_images: &mut WriteStorage<'_, UiImage>,
        ui_interactables: &mut WriteStorage<'_, Interactable>,
        hidden_propagates: &mut WriteStorage<'_, HiddenPropagate>,
        hierarchy: &ReadExpect<'_, ParentHierarchy>,
        now: Duration,
    ) {
        let transition_completed =
            (now - self.transition_began_at).as_millis() as f32 / MENU_FADE_OUT_DURATION_MS as f32;

        match self.transition_state {
            TransitionState::FadeOut => {
                let new_alpha = num::Float::max(0.0, 1.0 - transition_completed);

                for element_to_hide in &self.elements_to_hide {
                    let ui_entity = ui_finder.find_with_mut_transform(element_to_hide);
                    let (ui_entity, ui_transform) = if ui_entity.is_some() {
                        ui_entity.unwrap()
                    } else {
                        continue;
                    };

                    if *element_to_hide != UI_BACKGROUND_CONTAINER {
                        ui_transform.local_z = 0.5;
                    }
                    ui_interactables.remove(ui_entity);

                    if transition_completed > 1.0 {
                        hidden_propagates
                            .insert(ui_entity, HiddenPropagate)
                            .expect("Expected to insert HiddenPropagate component");
                    } else {
                        let hierarchy = if *element_to_hide == UI_BACKGROUND_CONTAINER {
                            None
                        } else {
                            Some(hierarchy)
                        };
                        Self::set_alpha_for(new_alpha, ui_entity, ui_texts, ui_images, hierarchy);
                    }
                }

                if transition_completed > 1.0 {
                    self.elements_to_hide.clear();
                    self.transition_state = TransitionState::FadeIn;
                    self.transition_began_at = now;
                }
            }
            TransitionState::FadeIn => {
                let new_alpha = num::Float::min(1.0, transition_completed);

                for element_to_show in &self.elements_to_show {
                    let ui_entity = ui_finder.find_with_mut_transform(element_to_show);
                    let (ui_entity, ui_transform) = if ui_entity.is_some() {
                        ui_entity.unwrap()
                    } else {
                        continue;
                    };

                    hidden_propagates.remove(ui_entity);

                    let hierarchy = if *element_to_show == UI_BACKGROUND_CONTAINER {
                        None
                    } else {
                        Some(hierarchy)
                    };
                    Self::set_alpha_for(new_alpha, ui_entity, ui_texts, ui_images, hierarchy);
                    if transition_completed > 1.0 {
                        if *element_to_show != UI_BACKGROUND_CONTAINER {
                            ui_transform.local_z = 1.0;
                        }
                        if self.mouse_reactive.contains(element_to_show) {
                            ui_interactables
                                .insert(ui_entity, Interactable)
                                .expect("Expected to insert Interactable component");
                        }
                    }
                }

                if transition_completed > 1.0 {
                    self.elements_to_show.clear();
                    self.transition_state = TransitionState::Still;
                }
            }
            TransitionState::Still => {}
        }

        if transition_completed > 1.0
            && self.elements_to_hide.is_empty()
            && self.elements_to_show.is_empty()
        {
            self.transition_state = TransitionState::Still;
            self.is_transitioning = false;
        }
    }

    fn set_alpha_for(
        new_alpha: f32,
        ui_entity: Entity,
        ui_texts: &mut WriteStorage<'_, UiText>,
        ui_images: &mut WriteStorage<'_, UiImage>,
        hierarchy: Option<&ReadExpect<'_, ParentHierarchy>>,
    ) {
        if let Some(ui_text) = ui_texts.get_mut(ui_entity) {
            ui_text.color[3] = new_alpha;
        } else if let Some(UiImage::SolidColor(ref mut color)) = ui_images.get_mut(ui_entity) {
            color[3] = new_alpha;
        }

        if let Some(hierarchy) = hierarchy {
            for ui_entity in hierarchy.children(ui_entity) {
                Self::set_alpha_for(new_alpha, *ui_entity, ui_texts, ui_images, Some(hierarchy))
            }
        }
    }
}

fn with_background(menu_elements: &[&'static str]) -> Vec<&'static str> {
    let mut elements = menu_elements.to_vec();
    elements.push(UI_BACKGROUND_CONTAINER);
    elements
}
