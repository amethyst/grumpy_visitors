use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::{Join, World},
    input::is_close_requested,
    prelude::{SimpleTrans, StateEvent, Trans},
    renderer::{camera::Projection, Camera},
    window::ScreenDimensions,
};
use winit::{ElementState, VirtualKeyCode};

use crate::application_settings::ApplicationSettings;

pub fn handle_window_event(world: &World, event: &StateEvent) -> Option<SimpleTrans> {
    let mut application_settings = world.write_resource::<ApplicationSettings>();
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
                    //                    let mut window_messages = world.write_resource::<WindowMessages>();
                    //                    let is_fullscreen = display.fullscreen;
                    //                    application_settings
                    //                        .save_fullscreen(!is_fullscreen)
                    //                        .expect("Failed to save settings");
                    //
                    //                    window_messages.send_command(move |window| {
                    //                        let monitor_id = if is_fullscreen {
                    //                            None
                    //                        } else {
                    //                            window.get_available_monitors().next()
                    //                        };
                    //                        window.set_fullscreen(monitor_id);
                    //                    });
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
                let mut cameras = world.write_storage::<Camera>();
                let mut transforms = world.write_storage::<Transform>();
                let (camera, camera_transform) =
                    (&mut cameras, &mut transforms).join().next().unwrap();
                let (screen_width, screen_height) = (size.width as f32, size.height as f32);

                camera.set_projection(Projection::orthographic(
                    0.0,
                    screen_width,
                    0.0,
                    screen_height,
                    0.1,
                    2000.0,
                ));
                camera_transform.set_translation(Vector3::new(
                    -screen_width / 2.0,
                    -screen_height / 2.0,
                    1.0,
                ));
            }

            _ => {}
        };
    }

    None
}
