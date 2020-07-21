use amethyst::{
    ecs::{Join, World},
    input::is_close_requested,
    prelude::{SimpleTrans, StateEvent, Trans, WorldExt},
    renderer::{camera::Projection, Camera},
    window::{MonitorIdent, ScreenDimensions, Window},
    winit::{self, ElementState},
};

use crate::settings::Settings;

// TODO: I don't like how this module looks, though I dunno why and how make it look better.
pub fn handle_window_event(world: &World, event: &StateEvent) -> Option<SimpleTrans> {
    let mut application_settings = world.fetch_mut::<Settings>();
    let display = application_settings.display();

    let toggle_fullscreen = application_settings.get_action_keycode("toggle_fullscreen");
    let log_dimensions = application_settings.get_action_keycode("log_dimensions");

    if let StateEvent::Window(event) = &event {
        if is_close_requested(&event) {
            return Some(Trans::Quit);
        }

        match event {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == ElementState::Released => {
                if input.virtual_keycode == toggle_fullscreen {
                    let window = world.fetch_mut::<Window>();

                    let monitor_id = if display.fullscreen.is_some() {
                        None
                    } else {
                        Some(window.get_current_monitor())
                    };

                    let fullscreen_monitor_ident = monitor_id
                        .clone()
                        .and_then(|id| MonitorIdent::from_monitor_id(&*window, id));
                    application_settings
                        .save_fullscreen(fullscreen_monitor_ident)
                        .expect("Failed to save settings");
                    window.set_fullscreen(monitor_id);
                } else if input.virtual_keycode == log_dimensions {
                    let screen_dimensions = world.fetch::<ScreenDimensions>();
                    println!(
                        "{}:{}",
                        screen_dimensions.width(),
                        screen_dimensions.height()
                    );
                }
            }

            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(size),
                ..
            } => {
                let hidpi = world.fetch::<ScreenDimensions>().hidpi_factor();
                let mut cameras = world.write_component::<Camera>();
                let camera = (&mut cameras).join().next().unwrap();
                let (screen_width, screen_height) =
                    ((size.width * hidpi) as f32, (size.height * hidpi) as f32);

                camera.set_projection(Projection::orthographic(
                    -screen_width / 2.0,
                    screen_width / 2.0,
                    -screen_height / 2.0,
                    screen_height / 2.0,
                    0.1,
                    2000.0,
                ));
            }

            _ => {}
        };
    }

    None
}
