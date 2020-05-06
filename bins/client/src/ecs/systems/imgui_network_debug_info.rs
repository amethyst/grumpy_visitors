use amethyst::ecs::{System, WriteExpect};

use crate::ecs::resources::DisplayDebugInfoSettings;

pub struct ImguiNetworkDebugInfoSystem;

impl<'s> System<'s> for ImguiNetworkDebugInfoSystem {
    type SystemData = ((), WriteExpect<'s, DisplayDebugInfoSettings>);

    fn run(&mut self, (_, mut display_debug_info_settings): Self::SystemData) {
        amethyst_imgui::with(|ui| {
            if display_debug_info_settings.display_network_debug_info {
                ui.show_demo_window(&mut display_debug_info_settings.display_network_debug_info);
            }
        });
    }
}
