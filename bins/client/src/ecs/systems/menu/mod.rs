mod hidden;
mod lobby;
mod main;
mod multiplayer_room;
mod restart;

use amethyst::{
    core::{HiddenPropagate, ParentHierarchy},
    ecs::{Entity, ReadExpect, System, SystemData, World, Write, WriteExpect, WriteStorage},
    shred::ResourceId,
    shrev::{EventChannel, ReaderId},
    ui::{Interactable, UiEvent, UiEventType, UiImage, UiText},
};
use lazy_static::lazy_static;

use std::{collections::VecDeque, time::Duration};

use gv_client_shared::ecs::resources::MultiplayerRoomState;
use gv_core::ecs::{
    resources::{net::MultiplayerGameState, GameEngineState, GameLevelState, NewGameEngineState},
    system_data::time::GameTimeService,
};

use crate::ecs::{
    resources::UiNetworkCommandResource,
    system_data::ui::UiFinderMut,
    systems::menu::{
        hidden::HiddenMenuScreen, lobby::LobbyMenuScreen, main::MainMenuScreen,
        multiplayer_room::MultiplayerRoomMenuScreen, restart::RestartMenuScreen,
    },
};

type MenuElement = &'static str;

const MENU_FADE_OUT_DURATION_MS: u64 = 500;
const CONTAINER_TAG: &str = "_container";
const BACKGROUND_TAG: &str = "_bg";
const MODAL_TAG: &str = "_modal";
const UI_MAIN_CONTAINER: &str = "ui_main_container";
const UI_LOADING_LABEL: &str = "ui_loading_label";

const UI_SINGLE_PLAYER_BUTTON: &str = "ui_single_player_button";
const UI_MULTIPLAYER_BUTTON: &str = "ui_multiplayer_button";
const UI_QUIT_BUTTON: &str = "ui_quit_button";

const UI_RESTART_BUTTON: &str = "ui_restart_button";
const UI_MAIN_MENU_BUTTON: &str = "ui_main_menu_button";

const UI_LOBBY_NICKNAME_LABEL: &str = "ui_lobby_nickname_label";
const UI_LOBBY_NICKNAME_FIELD: &str = "ui_lobby_nickname_field";
const UI_LOBBY_NICKNAME_EDITABLE: &str = "ui_lobby_nickname_editable";
const UI_LOBBY_HOST_IP_FIELD: &str = "ui_lobby_host_ip_field";
const UI_LOBBY_HOST_IP_EDITABLE: &str = "ui_lobby_host_ip_editable";
const UI_LOBBY_HOST_BUTTON: &str = "ui_lobby_host_button";
const UI_LOBBY_JOIN_IP_FIELD: &str = "ui_lobby_join_ip_field";
const UI_LOBBY_JOIN_IP_EDITABLE: &str = "ui_lobby_join_ip_editable";
const UI_LOBBY_JOIN_BUTTON: &str = "ui_lobby_join_button";

const UI_MP_ROOM_START_BUTTON: &str = "ui_start_multiplayer_button";
const UI_MP_ROOM_LOBBY_BUTTON: &str = "ui_back_to_lobby_button";
const UI_MP_ROOM_PLAYER1_CONTAINER: &str = "ui_mp_room_player1_container";
const UI_MP_ROOM_PLAYER1_BG: &str = "ui_mp_room_player1_bg";
const UI_MP_ROOM_PLAYER1_NUMBER: &str = "ui_mp_room_player1_number";
const UI_MP_ROOM_PLAYER1_NICKNAME: &str = "ui_mp_room_player1_nickname";
const UI_MP_ROOM_PLAYER1_KICK: &str = "ui_mp_room_player1_kick";
const UI_MP_ROOM_PLAYER2_CONTAINER: &str = "ui_mp_room_player2_container";
const UI_MP_ROOM_PLAYER2_BG: &str = "ui_mp_room_player2_bg";
const UI_MP_ROOM_PLAYER2_NUMBER: &str = "ui_mp_room_player2_number";
const UI_MP_ROOM_PLAYER2_NICKNAME: &str = "ui_mp_room_player2_nickname";
const UI_MP_ROOM_PLAYER2_KICK: &str = "ui_mp_room_player2_kick";
const UI_MP_ROOM_PLAYER3_CONTAINER: &str = "ui_mp_room_player3_container";
const UI_MP_ROOM_PLAYER3_BG: &str = "ui_mp_room_player3_bg";
const UI_MP_ROOM_PLAYER3_NUMBER: &str = "ui_mp_room_player3_number";
const UI_MP_ROOM_PLAYER3_NICKNAME: &str = "ui_mp_room_player3_nickname";
const UI_MP_ROOM_PLAYER3_KICK: &str = "ui_mp_room_player3_kick";
const UI_MP_ROOM_PLAYER4_CONTAINER: &str = "ui_mp_room_player4_container";
const UI_MP_ROOM_PLAYER4_BG: &str = "ui_mp_room_player4_bg";
const UI_MP_ROOM_PLAYER4_NUMBER: &str = "ui_mp_room_player4_number";
const UI_MP_ROOM_PLAYER4_NICKNAME: &str = "ui_mp_room_player4_nickname";
const UI_MP_ROOM_PLAYER4_KICK: &str = "ui_mp_room_player4_kick";

const UI_MODAL_BACKDROP_CONTAINER: &str = "ui_modal_backdrop_container";
const UI_MODAL_WINDOW_BORDER_CONTAINER: &str = "ui_modal_window_border_container";
const UI_MODAL_WINDOW_CONTAINER: &str = "ui_modal_window_container";
const UI_MODAL_TITLE: &str = "ui_modal_title";
const UI_MODAL_CONFIRM_BUTTON: &str = "ui_modal_confirm_button";

trait MenuScreen {
    fn elements_to_show(&self, system_data: &MenuSystemData) -> Vec<MenuElement>;

    fn elements_to_hide(&self, system_data: &MenuSystemData) -> Vec<MenuElement> {
        self.elements_to_show(system_data)
    }

    fn show(&mut self, _system_data: &mut MenuSystemData) {}

    fn value_changed(
        &mut self,
        _system_data: &mut MenuSystemData,
        _text_field_id: &str,
        _new_value: &str,
    ) {
    }

    fn update(
        &mut self,
        system_data: &mut MenuSystemData,
        button_pressed: Option<&str>,
        modal_window_id: Option<&str>,
    ) -> StateUpdate;
}

lazy_static! {
    static ref MAIN_MENU_ELEMENTS: &'static [&'static str] = &[
        UI_SINGLE_PLAYER_BUTTON,
        UI_MULTIPLAYER_BUTTON,
        UI_QUIT_BUTTON,
    ];
    static ref RESTART_MENU_ELEMENTS: &'static [&'static str] =
        &[UI_RESTART_BUTTON, UI_MAIN_MENU_BUTTON];
    static ref LOBBY_MENU_ELEMENTS: &'static [&'static str] = &[
        UI_LOBBY_NICKNAME_LABEL,
        UI_LOBBY_NICKNAME_FIELD,
        UI_LOBBY_NICKNAME_EDITABLE,
        UI_LOBBY_HOST_IP_FIELD,
        UI_LOBBY_HOST_IP_EDITABLE,
        UI_LOBBY_HOST_BUTTON,
        UI_LOBBY_JOIN_IP_FIELD,
        UI_LOBBY_JOIN_IP_EDITABLE,
        UI_LOBBY_JOIN_BUTTON,
        UI_MAIN_MENU_BUTTON,
    ];
    // TODO: implement and uncomment buttons.
    static ref MP_ROOM_MENU_ELEMENTS: &'static [&'static str] = &[
        UI_MP_ROOM_START_BUTTON,
        UI_MP_ROOM_LOBBY_BUTTON,
        UI_MP_ROOM_PLAYER1_CONTAINER,
        UI_MP_ROOM_PLAYER1_BG,
        UI_MP_ROOM_PLAYER1_NUMBER,
        UI_MP_ROOM_PLAYER1_NICKNAME,
        UI_MP_ROOM_PLAYER1_KICK,
        UI_MP_ROOM_PLAYER2_CONTAINER,
        UI_MP_ROOM_PLAYER2_BG,
        UI_MP_ROOM_PLAYER2_NUMBER,
        UI_MP_ROOM_PLAYER2_NICKNAME,
        UI_MP_ROOM_PLAYER2_KICK,
        UI_MP_ROOM_PLAYER3_CONTAINER,
        UI_MP_ROOM_PLAYER3_BG,
        UI_MP_ROOM_PLAYER3_NUMBER,
        UI_MP_ROOM_PLAYER3_NICKNAME,
        UI_MP_ROOM_PLAYER3_KICK,
        UI_MP_ROOM_PLAYER4_CONTAINER,
        UI_MP_ROOM_PLAYER4_BG,
        UI_MP_ROOM_PLAYER4_NUMBER,
        UI_MP_ROOM_PLAYER4_NICKNAME,
        UI_MP_ROOM_PLAYER4_KICK,
    ];
    static ref MODAL_WINDOW_ELEMENTS: &'static [&'static str] = &[
        UI_MODAL_BACKDROP_CONTAINER,
        UI_MODAL_WINDOW_BORDER_CONTAINER,
        UI_MODAL_WINDOW_CONTAINER,
        UI_MODAL_TITLE,
    ];
}

#[derive(SystemData)]
pub struct MenuSystemData<'s> {
    game_time_service: GameTimeService<'s>,
    ui_finder: UiFinderMut<'s>,
    hierarchy: ReadExpect<'s, ParentHierarchy>,
    game_engine_state: ReadExpect<'s, GameEngineState>,
    new_game_engine_state: WriteExpect<'s, NewGameEngineState>,
    game_level_state: WriteExpect<'s, GameLevelState>,
    ui_network_command: WriteExpect<'s, UiNetworkCommandResource>,
    multiplayer_room_state: ReadExpect<'s, MultiplayerRoomState>,
    multiplayer_game_state: ReadExpect<'s, MultiplayerGameState>,
    ui_events: Write<'s, EventChannel<UiEvent>>,
    ui_texts: WriteStorage<'s, UiText>,
    ui_images: WriteStorage<'s, UiImage>,
    ui_interactables: WriteStorage<'s, Interactable>,
    hidden_propagates: WriteStorage<'s, HiddenPropagate>,
}

pub struct MenuSystem {
    menu_screens: MenuScreens,
    modal_window_id: Option<String>,
    mouse_reactive: Vec<&'static str>,
    menu_screen_animations: VecDeque<MenuScreenAnimation>,
    event_readers: Option<ReaderId<UiEvent>>,
    menu_screen: GameMenuScreen,
    transition_state: TransitionState,
}

#[derive(Debug)]
struct MenuScreenAnimation {
    change_modal_title: Option<String>,
    started_at: Option<Duration>,
    elements_to_hide: Vec<&'static str>,
    elements_to_show: Vec<&'static str>,
}

struct MenuScreens {
    lobby_menu_screen: LobbyMenuScreen,
    main_menu_screen: MainMenuScreen,
    multiplayer_room_menu_screen: MultiplayerRoomMenuScreen,
    restart_menu_screen: RestartMenuScreen,
    hidden_menu_screen: HiddenMenuScreen,
}

impl MenuScreens {
    fn menu_screen(&mut self, screen: GameMenuScreen) -> Option<&mut dyn MenuScreen> {
        match screen {
            GameMenuScreen::LobbyMenu => Some(&mut self.lobby_menu_screen),
            GameMenuScreen::MainMenu => Some(&mut self.main_menu_screen),
            GameMenuScreen::MultiplayerRoomMenu => Some(&mut self.multiplayer_room_menu_screen),
            GameMenuScreen::RestartMenu => Some(&mut self.restart_menu_screen),
            GameMenuScreen::Hidden => Some(&mut self.hidden_menu_screen),
            GameMenuScreen::Loading => None,
        }
    }
}

enum StateUpdate {
    GameMenuUpdate {
        game_engine_state: Option<GameEngineState>,
        menu_screen: Option<GameMenuScreen>,
    },
    ShowModalWindow {
        id: String,
        title: String,
        show_confirmation: bool,
    },
    CustomAnimation {
        elements_to_hide: Vec<&'static str>,
        elements_to_show: Vec<&'static str>,
    },
    None,
}

impl StateUpdate {
    pub fn new_game_engine_state(game_engine_state: GameEngineState) -> Self {
        Self::GameMenuUpdate {
            game_engine_state: Some(game_engine_state),
            menu_screen: None,
        }
    }

    pub fn new_menu_screen(menu_screen: GameMenuScreen) -> Self {
        Self::GameMenuUpdate {
            game_engine_state: None,
            menu_screen: Some(menu_screen),
        }
    }
}

impl MenuSystem {
    pub fn new() -> Self {
        Self {
            menu_screens: MenuScreens {
                lobby_menu_screen: LobbyMenuScreen,
                main_menu_screen: MainMenuScreen,
                multiplayer_room_menu_screen: MultiplayerRoomMenuScreen::new(),
                restart_menu_screen: RestartMenuScreen,
                hidden_menu_screen: HiddenMenuScreen,
            },
            modal_window_id: None,
            mouse_reactive: vec![
                UI_SINGLE_PLAYER_BUTTON,
                UI_MULTIPLAYER_BUTTON,
                UI_QUIT_BUTTON,
                UI_RESTART_BUTTON,
                UI_MAIN_MENU_BUTTON,
                UI_LOBBY_NICKNAME_EDITABLE,
                UI_LOBBY_HOST_IP_EDITABLE,
                UI_LOBBY_HOST_BUTTON,
                UI_LOBBY_JOIN_IP_EDITABLE,
                UI_LOBBY_JOIN_BUTTON,
                UI_MP_ROOM_START_BUTTON,
                UI_MP_ROOM_LOBBY_BUTTON,
                UI_MP_ROOM_PLAYER1_KICK,
                UI_MP_ROOM_PLAYER2_KICK,
                UI_MP_ROOM_PLAYER3_KICK,
                UI_MP_ROOM_PLAYER4_KICK,
                UI_MODAL_CONFIRM_BUTTON,
            ],
            menu_screen_animations: VecDeque::new(),
            event_readers: None,
            menu_screen: GameMenuScreen::Loading,
            transition_state: TransitionState::Still,
        }
    }
}

impl Default for MenuSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum GameMenuScreen {
    Loading,
    MainMenu,
    RestartMenu,
    LobbyMenu,
    MultiplayerRoomMenu,
    Hidden,
}

#[derive(Debug)]
enum TransitionState {
    FadeIn,
    FadeOut,
    Still,
}

#[derive(Clone, Copy)]
struct AlphaTransition {
    normal_alpha: f32,
    current_step: f32,
    is_fade_in: bool,
}

impl AlphaTransition {
    fn modify(self, current_alpha: &mut [f32; 4]) -> bool {
        let goal = if self.is_fade_in {
            self.normal_alpha
        } else {
            0.0
        };
        if (current_alpha[3] - goal).abs() < 0.0001 {
            return false;
        }
        current_alpha[3] = if self.is_fade_in {
            let new_alpha = self.normal_alpha * self.current_step;
            num::Float::max(new_alpha, current_alpha[3])
        } else {
            let new_alpha = self.normal_alpha * (1.0 - self.current_step);
            num::Float::min(new_alpha, current_alpha[3])
        };
        true
    }
}

impl<'s> System<'s> for MenuSystem {
    type SystemData = MenuSystemData<'s>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        let now = system_data
            .game_time_service
            .engine_time()
            .absolute_real_time();

        let event_readers = self
            .event_readers
            .get_or_insert_with(|| system_data.ui_events.register_reader());

        let mut button_pressed = None;
        let mut value_changed = None;
        for event in system_data.ui_events.read(event_readers) {
            let target_id = system_data.ui_finder.get_id_by_entity(event.target);
            log::trace!("{:?}: {:?}", target_id, event);

            match &event.event_type {
                UiEventType::Click => {
                    button_pressed = target_id;
                    // Prevent double-clicking.
                    system_data.ui_interactables.remove(event.target);
                }
                UiEventType::ValueChange => {
                    let text_field_id = system_data
                        .ui_finder
                        .get_id_by_entity(event.target)
                        .expect("Expected an edited text field");
                    let new_value = system_data
                        .ui_finder
                        .get_ui_text(&system_data.ui_texts, &text_field_id)
                        .expect("Expected an edited text field");
                    value_changed = Some((text_field_id, new_value.clone()));
                }
                _ => {}
            };
        }

        if let Some(ui_loading) = system_data.ui_finder.find(UI_LOADING_LABEL) {
            let dots_count = (now.as_secs() as usize + 2) % 3 + 1;
            let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
            let ui_loading_text = system_data.ui_texts.get_mut(ui_loading).unwrap();
            ui_loading_text.text = "Loading".to_owned() + &dots;
        }

        self.run_fade_animation(&mut system_data, now);

        // Pass the events to the active menu screen handler.
        let state_update = match (&*system_data.game_engine_state, self.menu_screen) {
            (GameEngineState::Menu, GameMenuScreen::Loading) => {
                StateUpdate::new_menu_screen(GameMenuScreen::MainMenu)
            }
            (GameEngineState::Menu, menu_screen) => {
                let menu_screen = self
                    .menu_screens
                    .menu_screen(menu_screen)
                    .expect("Expected a menu screen for GameEngineState::Menu");
                if let Some((text_field_id, value_changed)) = value_changed {
                    menu_screen.value_changed(&mut system_data, &text_field_id, &value_changed);
                }
                menu_screen.update(
                    &mut system_data,
                    button_pressed.as_deref(),
                    self.modal_window_id.as_deref(),
                )
            }
            (GameEngineState::Playing, menu_screen) if menu_screen != GameMenuScreen::Hidden => {
                StateUpdate::new_menu_screen(GameMenuScreen::Hidden)
            }
            (GameEngineState::Playing, _) => self.menu_screens.hidden_menu_screen.update(
                &mut system_data,
                button_pressed.as_deref(),
                self.modal_window_id.as_deref(),
            ),
            _ => StateUpdate::None,
        };

        let (change_modal_title, mut elements_to_hide, elements_to_show) = match state_update {
            StateUpdate::GameMenuUpdate {
                game_engine_state,
                menu_screen,
            } => {
                if let Some(new_game_engine_state) = game_engine_state {
                    *system_data.new_game_engine_state = NewGameEngineState(new_game_engine_state);
                }
                if let Some(new_menu_screen) = menu_screen {
                    let current_menu_screen = self.menu_screen;
                    let mut elements_to_hide = if let GameMenuScreen::Loading = current_menu_screen
                    {
                        vec![UI_LOADING_LABEL]
                    } else {
                        self.menu_screens
                            .menu_screen(current_menu_screen)
                            .map(|menu_screen| {
                                let elements = menu_screen.elements_to_hide(&system_data);
                                if let GameMenuScreen::Hidden = new_menu_screen {
                                    with_background(&elements)
                                } else {
                                    elements
                                }
                            })
                            .unwrap_or_default()
                    };
                    let elements_to_show = self
                        .menu_screens
                        .menu_screen(new_menu_screen)
                        .map(|menu_screen| {
                            let elements = menu_screen.elements_to_show(&system_data);
                            if let GameMenuScreen::Hidden = current_menu_screen {
                                with_background(&elements)
                            } else {
                                elements
                            }
                        })
                        .unwrap_or_default();
                    self.menu_screen = new_menu_screen;
                    if let Some(menu_screen) = self.menu_screens.menu_screen(new_menu_screen) {
                        menu_screen.show(&mut system_data);
                    }
                    self.modal_window_id = None;
                    elements_to_hide.append(&mut modal_window_with_confirmation());
                    (None, elements_to_hide, elements_to_show)
                } else {
                    (None, vec![], vec![])
                }
            }
            StateUpdate::ShowModalWindow {
                id,
                title,
                show_confirmation,
            } => {
                log::info!(
                    "Show modal window {}, show confirmation: {}",
                    id,
                    show_confirmation
                );
                self.modal_window_id = Some(id);
                if show_confirmation {
                    (Some(title), vec![], modal_window_with_confirmation())
                } else {
                    (Some(title), vec![UI_MODAL_CONFIRM_BUTTON], modal_window())
                }
            }
            StateUpdate::CustomAnimation {
                elements_to_hide,
                elements_to_show,
            } => (None, elements_to_hide, elements_to_show),
            StateUpdate::None => (None, vec![], vec![]),
        };

        if self.modal_window_id.is_some() {
            if let Some(UI_MODAL_CONFIRM_BUTTON) = button_pressed.as_deref() {
                self.modal_window_id = None;
                elements_to_hide.append(&mut modal_window_with_confirmation());
            }
        }

        if !elements_to_show.is_empty() || !elements_to_hide.is_empty() {
            self.add_fade_animation(change_modal_title, elements_to_hide, elements_to_show);
        }
    }
}

impl MenuSystem {
    fn add_fade_animation(
        &mut self,
        change_modal_title: Option<String>,
        elements_to_hide: Vec<&'static str>,
        elements_to_show: Vec<&'static str>,
    ) {
        self.menu_screen_animations.push_back(MenuScreenAnimation {
            change_modal_title,
            started_at: None,
            elements_to_hide,
            elements_to_show,
        })
    }

    #[allow(clippy::cognitive_complexity)]
    fn run_fade_animation(
        &mut self,
        system_data: &mut <Self as System>::SystemData,
        now: Duration,
    ) {
        let menu_screen_animation = self.menu_screen_animations.get_mut(0);
        if menu_screen_animation.is_none() {
            return;
        }
        let menu_screen_animation = menu_screen_animation.unwrap();

        if let Some(change_modal_title) = menu_screen_animation.change_modal_title.clone() {
            *system_data
                .ui_finder
                .get_ui_text_mut(&mut system_data.ui_texts, UI_MODAL_TITLE)
                .unwrap() = change_modal_title;
        }

        if menu_screen_animation.started_at.is_none() {
            if let TransitionState::Still = self.transition_state {
            } else {
                panic!("Transition state must be Still before starting a new transition");
            }
            if !menu_screen_animation.elements_to_hide.is_empty() {
                self.transition_state = TransitionState::FadeOut;
            } else if !menu_screen_animation.elements_to_show.is_empty() {
                self.transition_state = TransitionState::FadeIn;
            } else {
                panic!("There's no elements to show or hide");
            }
            log::debug!(
                "Starting a new menu screen animation at {}s ({:?}): {:?}",
                now.as_secs_f32(),
                self.transition_state,
                menu_screen_animation
            );
            menu_screen_animation.started_at = Some(now);
        }
        let started_at = menu_screen_animation.started_at.unwrap();

        let transition_completed =
            (now - started_at).as_millis() as f32 / MENU_FADE_OUT_DURATION_MS as f32;

        match self.transition_state {
            TransitionState::FadeOut => {
                let mut modified = 0;
                for element_to_hide in &menu_screen_animation.elements_to_hide {
                    let alpha_transition = if element_to_hide.contains(MODAL_TAG) {
                        if *element_to_hide == UI_MODAL_BACKDROP_CONTAINER {
                            AlphaTransition {
                                normal_alpha: 0.7,
                                current_step: transition_completed,
                                is_fade_in: false,
                            }
                        } else {
                            AlphaTransition {
                                normal_alpha: 1.0,
                                current_step: 1.0,
                                is_fade_in: false,
                            }
                        }
                    } else {
                        AlphaTransition {
                            normal_alpha: 1.0,
                            current_step: transition_completed,
                            is_fade_in: false,
                        }
                    };
                    let ui_entity = system_data
                        .ui_finder
                        .find_with_mut_transform(element_to_hide);
                    let is_container = element_to_hide.contains(CONTAINER_TAG);
                    let (ui_entity, ui_transform) = if let Some(ui_entity) = ui_entity {
                        ui_entity
                    } else {
                        log::warn!("Couldn't find a UI entity: {}", element_to_hide);
                        continue;
                    };

                    if !is_container && !element_to_hide.contains(BACKGROUND_TAG) {
                        ui_transform.local_z = 0.5;
                    } else if *element_to_hide == UI_MODAL_BACKDROP_CONTAINER
                        && transition_completed >= 1.0
                    {
                        ui_transform.local_z = 100.0;
                    }
                    system_data.ui_interactables.remove(ui_entity);

                    let hierarchy = if is_container {
                        None
                    } else {
                        Some(&system_data.hierarchy)
                    };
                    if transition_completed >= 1.0
                        || (element_to_hide.contains(MODAL_TAG)
                            && *element_to_hide != UI_MODAL_BACKDROP_CONTAINER)
                    {
                        system_data
                            .hidden_propagates
                            .insert(ui_entity, HiddenPropagate::new())
                            .expect("Expected to insert HiddenPropagate component");
                    } else {
                        modified += Self::set_alpha_for(
                            alpha_transition,
                            ui_entity,
                            &mut system_data.ui_texts,
                            &mut system_data.ui_images,
                            hierarchy,
                        );
                    }
                }

                if transition_completed >= 1.0 || (modified == 0 && transition_completed != 0.0) {
                    menu_screen_animation.elements_to_hide.clear();
                    self.transition_state = TransitionState::FadeIn;
                    menu_screen_animation.started_at = Some(now);
                    log::debug!(
                        "Starting a new menu screen animation at {}s ({:?}): {:?}",
                        now.as_secs_f32(),
                        self.transition_state,
                        menu_screen_animation
                    );
                }
            }
            TransitionState::FadeIn => {
                let mut modified = 0;
                for element_to_show in &menu_screen_animation.elements_to_show {
                    let alpha_transition = if element_to_show.contains(MODAL_TAG) {
                        if *element_to_show == UI_MODAL_BACKDROP_CONTAINER {
                            AlphaTransition {
                                normal_alpha: 0.7,
                                current_step: transition_completed,
                                is_fade_in: true,
                            }
                        } else {
                            AlphaTransition {
                                normal_alpha: 1.0,
                                current_step: 1.0,
                                is_fade_in: true,
                            }
                        }
                    } else {
                        AlphaTransition {
                            normal_alpha: 1.0,
                            current_step: transition_completed,
                            is_fade_in: true,
                        }
                    };
                    let ui_entity = system_data
                        .ui_finder
                        .find_with_mut_transform(element_to_show);
                    let is_container = element_to_show.contains(CONTAINER_TAG);
                    let (ui_entity, ui_transform) = if let Some(ui_entity) = ui_entity {
                        ui_entity
                    } else {
                        log::warn!("Couldn't find a UI entity: {}", element_to_show);
                        continue;
                    };

                    system_data.hidden_propagates.remove(ui_entity);
                    let hierarchy = if is_container {
                        None
                    } else {
                        Some(&system_data.hierarchy)
                    };
                    modified += Self::set_alpha_for(
                        alpha_transition,
                        ui_entity,
                        &mut system_data.ui_texts,
                        &mut system_data.ui_images,
                        hierarchy,
                    );
                    if *element_to_show == UI_MODAL_BACKDROP_CONTAINER {
                        ui_transform.local_z = 150.0;
                    }
                    if transition_completed >= 1.0 {
                        if !is_container && !element_to_show.contains(BACKGROUND_TAG) {
                            ui_transform.local_z = 1.0;
                        }
                        if self.mouse_reactive.contains(element_to_show) {
                            system_data
                                .ui_interactables
                                .insert(ui_entity, Interactable)
                                .expect("Expected to insert Interactable component");
                        }
                    }
                }

                if transition_completed >= 1.0 || (modified == 0 && transition_completed != 0.0) {
                    menu_screen_animation.elements_to_show.clear();
                    self.transition_state = TransitionState::Still;
                }
            }
            TransitionState::Still => {}
        }

        if menu_screen_animation.elements_to_hide.is_empty()
            && menu_screen_animation.elements_to_show.is_empty()
        {
            self.transition_state = TransitionState::Still;
            self.menu_screen_animations.pop_front();
        }
    }

    fn set_alpha_for(
        new_alpha: AlphaTransition,
        ui_entity: Entity,
        ui_texts: &mut WriteStorage<UiText>,
        ui_images: &mut WriteStorage<UiImage>,
        hierarchy: Option<&ReadExpect<ParentHierarchy>>,
    ) -> u16 {
        let mut modified = 0;
        if let Some(ui_text) = ui_texts.get_mut(ui_entity) {
            modified += new_alpha.modify(&mut ui_text.color) as u16;
        } else if let Some(UiImage::SolidColor(ref mut color)) = ui_images.get_mut(ui_entity) {
            modified += new_alpha.modify(color) as u16;
        }

        if let Some(hierarchy) = hierarchy {
            for ui_entity in hierarchy.children(ui_entity) {
                modified += Self::set_alpha_for(
                    new_alpha,
                    *ui_entity,
                    ui_texts,
                    ui_images,
                    Some(hierarchy),
                );
            }
        }

        modified
    }
}

fn with_background(menu_elements: &[MenuElement]) -> Vec<MenuElement> {
    let mut elements = menu_elements.to_vec();
    elements.push(UI_MAIN_CONTAINER);
    elements
}

fn modal_window() -> Vec<MenuElement> {
    MODAL_WINDOW_ELEMENTS.to_vec()
}

fn modal_window_with_confirmation() -> Vec<MenuElement> {
    let mut elements = MODAL_WINDOW_ELEMENTS.to_vec();
    elements.push(UI_MODAL_CONFIRM_BUTTON);
    elements
}
