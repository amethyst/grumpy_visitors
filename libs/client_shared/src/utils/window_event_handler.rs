use amethyst::{
    ecs::{Join, World},
    input::is_close_requested,
    prelude::{SimpleTrans, StateEvent, Trans},
    renderer::{camera::Projection, Camera},
    window::{MonitorIdent, ScreenDimensions, Window},
    winit::{self, ElementState, VirtualKeyCode},
};

use crate::settings::Settings;

// TODO: I don't like how this module looks, though I dunno why and how make it look better.
pub fn handle_window_event(world: &World, event: &StateEvent) -> Option<SimpleTrans> {
    let mut application_settings = world.write_resource::<Settings>();
    let display = application_settings.display();

    if let StateEvent::Window(event) = &event {
        if is_close_requested(&event) {
            return Some(Trans::Quit);
        }

        match event {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == ElementState::Released => match input.virtual_keycode {
                Some(VirtualKeyCode::F11) => {
                    let window = world.write_resource::<Window>();

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
                }
                Some(VirtualKeyCode::F10) => {
                    let screen_dimensions = world.read_resource::<ScreenDimensions>();
                    println!(
                        "{}:{}",
                        screen_dimensions.width(),
                        screen_dimensions.height()
                    );
                }
                _ => {}
            },

            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(size),
                ..
            } => {
                let hidpi = world.read_resource::<ScreenDimensions>().hidpi_factor();
                let mut cameras = world.write_storage::<Camera>();
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
