mod lobby;
mod main;
mod multiplayer_room;
mod restart;

use amethyst::{
    core::{Hidden, HiddenPropagate, ParentHierarchy},
    ecs::{Entity, ReadExpect, System, SystemData, World, Write, WriteExpect, WriteStorage},
    network::simulation::laminar::LaminarSocketResource,
    shred::ResourceId,
    shrev::{EventChannel, ReaderId},
    ui::{Interactable, UiEvent, UiEventType, UiImage, UiText},
};
use lazy_static::lazy_static;

use std::time::Duration;

use gv_client_shared::ecs::resources::MultiplayerRoomState;
use gv_core::ecs::{
    resources::{net::MultiplayerGameState, GameEngineState, GameLevelState, NewGameEngineState},
    system_data::time::GameTimeService,
};

use crate::ecs::{
    resources::ServerCommand,
    system_data::ui::UiFinderMut,
    systems::menu::{
        lobby::LobbyMenuScreen, main::MainMenuScreen, multiplayer_room::MultiplayerRoomMenuScreen,
        restart::RestartMenuScreen,
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

const UI_MODAL_BACKGROUND: &str = "ui_modal_background";
const UI_MODAL_WINDOW_BORDER: &str = "ui_modal_window_border";
const UI_MODAL_WINDOW: &str = "ui_modal_window";
const UI_MODAL_TITLE: &str = "ui_modal_title";
const UI_MODAL_CONFIRM_BUTTON: &str = "ui_modal_confirm_button";

trait MenuScreen {
    fn elements_to_show(&self, system_data: &MenuSystemData) -> Vec<MenuElement>;

    fn elements_to_hide(&self, system_data: &MenuSystemData) -> Vec<MenuElement> {
        self.elements_to_show(system_data)
    }

    fn process_events(
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
        UI_MP_ROOM_PLAYER2_CONTAINER,
        UI_MP_ROOM_PLAYER2_BG,
        UI_MP_ROOM_PLAYER2_NUMBER,
        UI_MP_ROOM_PLAYER2_NICKNAME,
        // UI_MP_ROOM_PLAYER2_KICK,
        UI_MP_ROOM_PLAYER3_CONTAINER,
        UI_MP_ROOM_PLAYER3_BG,
        UI_MP_ROOM_PLAYER3_NUMBER,
        UI_MP_ROOM_PLAYER3_NICKNAME,
        // UI_MP_ROOM_PLAYER3_KICK,
        UI_MP_ROOM_PLAYER4_CONTAINER,
        UI_MP_ROOM_PLAYER4_BG,
        UI_MP_ROOM_PLAYER4_NUMBER,
        UI_MP_ROOM_PLAYER4_NICKNAME,
        // UI_MP_ROOM_PLAYER4_KICK,
    ];
    static ref MODAL_WINDOW_ELEMENTS: &'static [&'static str] = &[
        UI_MODAL_BACKGROUND,
        UI_MODAL_WINDOW_BORDER,
        UI_MODAL_WINDOW,
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
    game_level_state: ReadExpect<'s, GameLevelState>,
    server_command: WriteExpect<'s, ServerCommand>,
    multiplayer_room_state: WriteExpect<'s, MultiplayerRoomState>,
    multiplayer_game_state: WriteExpect<'s, MultiplayerGameState>,
    laminar_socket: WriteExpect<'s, LaminarSocketResource>,
    ui_events: Write<'s, EventChannel<UiEvent>>,
    ui_texts: WriteStorage<'s, UiText>,
    ui_images: WriteStorage<'s, UiImage>,
    ui_interactables: WriteStorage<'s, Interactable>,
    hidden: WriteStorage<'s, Hidden>,
    hidden_propagates: WriteStorage<'s, HiddenPropagate>,
}

pub struct MenuSystem {
    menu_screens: MenuScreens,
    modal_window_id: Option<String>,
    elements_to_hide: Vec<&'static str>,
    elements_to_show: Vec<&'static str>,
    mouse_reactive: Vec<&'static str>,
    is_transitioning: bool,
    transition_began_at: Duration,
    event_readers: Option<ReaderId<UiEvent>>,
    menu_screen: GameMenuScreen,
    transition_state: TransitionState,
}

struct MenuScreens {
    lobby_menu_screen: LobbyMenuScreen,
    main_menu_screen: MainMenuScreen,
    multiplayer_room_menu_screen: MultiplayerRoomMenuScreen,
    restart_menu_screen: RestartMenuScreen,
}

impl MenuScreens {
    fn menu_screen(&mut self, screen: GameMenuScreen) -> Option<&mut dyn MenuScreen> {
        match screen {
            GameMenuScreen::LobbyMenu => Some(&mut self.lobby_menu_screen),
            GameMenuScreen::MainMenu => Some(&mut self.main_menu_screen),
            GameMenuScreen::MultiplayerRoomMenu => Some(&mut self.multiplayer_room_menu_screen),
            GameMenuScreen::RestartMenu => Some(&mut self.restart_menu_screen),
            _ => None,
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
                multiplayer_room_menu_screen: MultiplayerRoomMenuScreen,
                restart_menu_screen: RestartMenuScreen,
            },
            modal_window_id: None,
            elements_to_hide: Vec::new(),
            elements_to_show: Vec::new(),
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
                UI_MP_ROOM_PLAYER2_KICK,
                UI_MP_ROOM_PLAYER3_KICK,
                UI_MP_ROOM_PLAYER4_KICK,
                UI_MODAL_CONFIRM_BUTTON,
            ],
            is_transitioning: false,
            transition_began_at: Duration::new(0, 0),
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

enum TransitionState {
    FadeIn,
    FadeOut,
    Still,
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
        for event in system_data.ui_events.read(event_readers) {
            if let UiEventType::Click = event.event_type {
                button_pressed = system_data.ui_finder.get_id_by_entity(event.target);
            }
        }

        if let Some(ui_loading) = system_data.ui_finder.find(UI_LOADING_LABEL) {
            let dots_count = (now.as_secs() as usize + 2) % 3 + 1;
            let dots = std::iter::repeat(".").take(dots_count).collect::<String>();
            let ui_loading_text = system_data.ui_texts.get_mut(ui_loading).unwrap();
            ui_loading_text.text = "Loading".to_owned() + &dots;
        }

        self.run_fade_animation(&mut system_data, now);

        let state_update = match (&*system_data.game_engine_state, self.menu_screen) {
            (GameEngineState::Menu, GameMenuScreen::Loading) => {
                StateUpdate::new_menu_screen(GameMenuScreen::LobbyMenu)
            }
            (GameEngineState::Menu, menu_screen) => {
                let menu_screen = self.menu_screens.menu_screen(menu_screen).unwrap();
                menu_screen.process_events(
                    &mut system_data,
                    button_pressed.as_ref().map(std::string::String::as_str),
                    self.modal_window_id
                        .as_ref()
                        .map(std::string::String::as_str),
                )
            }
            (GameEngineState::Playing, menu_screen) if menu_screen != GameMenuScreen::Hidden => {
                StateUpdate::new_menu_screen(GameMenuScreen::Hidden)
            }
            (GameEngineState::Playing, _) if system_data.game_level_state.is_over => {
                StateUpdate::GameMenuUpdate {
                    game_engine_state: Some(GameEngineState::Menu),
                    menu_screen: Some(GameMenuScreen::RestartMenu),
                }
            }
            _ => StateUpdate::None,
        };

        let (elements_to_show, mut elements_to_hide) = match state_update {
            StateUpdate::GameMenuUpdate {
                game_engine_state,
                menu_screen,
            } => {
                if let Some(new_game_engine_state) = game_engine_state {
                    *system_data.new_game_engine_state = NewGameEngineState(new_game_engine_state);
                }
                if let Some(new_menu_screen) = menu_screen {
                    let current_menu_screen = self.menu_screen;
                    let elements_to_hide = if let GameMenuScreen::Loading = current_menu_screen {
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
                    (elements_to_show, elements_to_hide)
                } else {
                    (vec![], vec![])
                }
            }
            StateUpdate::ShowModalWindow {
                id,
                title,
                show_confirmation,
            } => {
                self.modal_window_id = Some(id);
                *system_data
                    .ui_finder
                    .get_ui_text_mut(&mut system_data.ui_texts, UI_MODAL_TITLE)
                    .unwrap() = title;
                let elements_to_show = if show_confirmation {
                    modal_window_with_confirmation()
                } else {
                    modal_window()
                };
                (elements_to_show, vec![])
            }
            StateUpdate::None => (vec![], vec![]),
        };

        if self.modal_window_id.is_some() {
            if let Some(UI_MODAL_CONFIRM_BUTTON) =
                button_pressed.as_ref().map(std::string::String::as_str)
            {
                self.modal_window_id = None;
                elements_to_hide.append(&mut modal_window_with_confirmation());
            }
        }

        if !elements_to_show.is_empty() || !elements_to_hide.is_empty() {
            self.set_fade_animation(now, elements_to_hide, elements_to_show);
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
            panic!("Transition state must be Still before starting a new transition");
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
        system_data: &mut <Self as System>::SystemData,
        now: Duration,
    ) {
        let transition_completed =
            (now - self.transition_began_at).as_millis() as f32 / MENU_FADE_OUT_DURATION_MS as f32;

        match self.transition_state {
            TransitionState::FadeOut => {
                let new_alpha = num::Float::max(0.0, 1.0 - transition_completed);

                for element_to_hide in &self.elements_to_hide {
                    let new_alpha = if element_to_hide.contains(MODAL_TAG) {
                        if *element_to_hide == UI_MODAL_BACKGROUND {
                            new_alpha * 0.7
                        } else {
                            0.0
                        }
                    } else {
                        new_alpha
                    };
                    let ui_entity = system_data
                        .ui_finder
                        .find_with_mut_transform(element_to_hide);
                    let is_container = element_to_hide.contains(CONTAINER_TAG);
                    let (ui_entity, ui_transform) = if let Some(ui_entity) = ui_entity {
                        ui_entity
                    } else {
                        continue;
                    };

                    if !is_container
                        && !element_to_hide.contains(BACKGROUND_TAG)
                        && *element_to_hide != UI_MODAL_BACKGROUND
                    {
                        ui_transform.local_z = 0.5;
                    } else if *element_to_hide == UI_MODAL_BACKGROUND && transition_completed > 1.0
                    {
                        ui_transform.local_z = 100.0;
                    }
                    system_data.ui_interactables.remove(ui_entity);

                    let hierarchy = if is_container {
                        None
                    } else {
                        Some(&system_data.hierarchy)
                    };
                    if transition_completed > 1.0
                        || (element_to_hide.contains(MODAL_TAG)
                            && *element_to_hide != UI_MODAL_BACKGROUND)
                    {
                        if is_container {
                            system_data
                                .hidden
                                .insert(ui_entity, Hidden)
                                .expect("Expected to insert Hidden component");
                        } else {
                            system_data
                                .hidden_propagates
                                .insert(ui_entity, HiddenPropagate)
                                .expect("Expected to insert HiddenPropagate component");
                        }
                    } else {
                        Self::set_alpha_for(
                            new_alpha,
                            ui_entity,
                            &mut system_data.ui_texts,
                            &mut system_data.ui_images,
                            hierarchy,
                        );
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
                    let new_alpha = if element_to_show.contains(MODAL_TAG) {
                        if *element_to_show == UI_MODAL_BACKGROUND {
                            new_alpha * 0.7
                        } else {
                            1.0
                        }
                    } else {
                        new_alpha
                    };
                    let ui_entity = system_data
                        .ui_finder
                        .find_with_mut_transform(element_to_show);
                    let is_container = element_to_show.contains(CONTAINER_TAG);
                    let (ui_entity, ui_transform) = if let Some(ui_entity) = ui_entity {
                        ui_entity
                    } else {
                        continue;
                    };

                    let hierarchy = if is_container {
                        system_data.hidden.remove(ui_entity);
                        None
                    } else {
                        system_data.hidden_propagates.remove(ui_entity);
                        Some(&system_data.hierarchy)
                    };
                    Self::set_alpha_for(
                        new_alpha,
                        ui_entity,
                        &mut system_data.ui_texts,
                        &mut system_data.ui_images,
                        hierarchy,
                    );
                    if *element_to_show == UI_MODAL_BACKGROUND {
                        ui_transform.local_z = 150.0;
                    }
                    if transition_completed > 1.0 {
                        if !is_container
                            && !element_to_show.contains(BACKGROUND_TAG)
                            && *element_to_show != UI_MODAL_BACKGROUND
                        {
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
        ui_texts: &mut WriteStorage<UiText>,
        ui_images: &mut WriteStorage<UiImage>,
        hierarchy: Option<&ReadExpect<ParentHierarchy>>,
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
