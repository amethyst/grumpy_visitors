use amethyst::{
    config::Config,
    input::{Bindings, Button, StringBindings},
    window::{DisplayConfig, MonitorIdent},
    winit::VirtualKeyCode,
};
use directories::ProjectDirs;
use ron::ser::PrettyConfig;

use std::{fs, path::PathBuf};

static DEFAULT_BINDINGS_CONFIG_BYTES: &[u8] =
    include_bytes!("../../../resources/bindings_config.ron");

static DEFAULT_DISPLAY_CONFIG_BYTES: &[u8] =
    include_bytes!("../../../resources/display_config.ron");

pub struct Settings {
    project_dirs: ProjectDirs,
    bindings: Bindings<StringBindings>,
    display: DisplayConfig,
}

impl Settings {
    pub fn new() -> amethyst::Result<Self> {
        let project_dirs = ProjectDirs::from("", "Psychedelic Donkey", "Grumpy Visitors")
            .expect("Failed to get the project directory");
        fs::create_dir_all(project_dirs.config_dir())?;

        let default_bindings =
            Bindings::<StringBindings>::load_bytes(DEFAULT_BINDINGS_CONFIG_BYTES)?;

        let bindings_config_path = bindings_config_path(&project_dirs);
        let bindings = {
            let mut needs_update = false;
            let mut bindings = Bindings::load(bindings_config_path.as_path()).or_else(
                |_| -> amethyst::Result<Bindings<StringBindings>> {
                    needs_update = true;
                    Ok(default_bindings.clone())
                },
            )?;

            if needs_update {
                fs::write(
                    bindings_config_path,
                    ron::ser::to_string_pretty(&bindings, PrettyConfig::default())?,
                )?;
                bindings
            } else {
                // Updating possible missing actions.
                for axis in default_bindings.axes() {
                    if bindings.axis(axis).is_none() {
                        log::warn!("Found missing axis bindings, updating the config");
                        needs_update = true;
                        bindings
                            .insert_axis(axis, default_bindings.axis(axis).unwrap().clone())
                            .expect("Expected to insert a missing axis entry");
                    }
                }

                for action in default_bindings.actions() {
                    if bindings.action_bindings(action).next().is_none() {
                        log::warn!("Found missing actions bindings, updating the config");
                        needs_update = true;
                        for default_binding in default_bindings.action_bindings(action) {
                            bindings
                                .insert_action_binding(
                                    action.clone(),
                                    default_binding.iter().cloned(),
                                )
                                .expect("Expected to insert a missing action entry");
                        }
                    }
                }

                if needs_update {
                    fs::write(
                        bindings_config_path,
                        ron::ser::to_string_pretty(&bindings, PrettyConfig::default())?,
                    )?;
                }

                bindings
            }
        };

        let display_config_path = display_config_path(&project_dirs);
        let display = DisplayConfig::load(display_config_path.as_path()).or_else(
            move |_| -> amethyst::Result<DisplayConfig> {
                let display = DisplayConfig::load_bytes(DEFAULT_DISPLAY_CONFIG_BYTES)?;
                fs::write(
                    display_config_path,
                    ron::ser::to_string_pretty(&display, PrettyConfig::default())?,
                )?;
                Ok(display)
            },
        )?;

        Ok(Self {
            project_dirs,
            bindings,
            display,
        })
    }

    pub fn bindings(&self) -> &Bindings<StringBindings> {
        &self.bindings
    }

    pub fn get_action_keycode(&self, action: &'static str) -> Option<VirtualKeyCode> {
        if let Some(button) = self.bindings.action_bindings(action).next() {
            match button[0] {
                Button::Key(key) => Some(key),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn display(&self) -> &DisplayConfig {
        &self.display
    }

    #[allow(dead_code)]
    pub fn save_resolution(&mut self, dimensions: (u32, u32)) -> amethyst::Result<()> {
        self.display.dimensions = Some(dimensions);
        self.save_display()
    }

    pub fn save_fullscreen(&mut self, fullscreen: Option<MonitorIdent>) -> amethyst::Result<()> {
        self.display.fullscreen = fullscreen;
        self.save_display()
    }

    #[allow(dead_code)]
    fn save_bindings(&mut self) -> amethyst::Result<()> {
        fs::create_dir_all(self.project_dirs.config_dir())?;
        fs::write(
            self.bindings_config_path(),
            ron::ser::to_string_pretty(&self.bindings, PrettyConfig::default())?,
        )?;
        Ok(())
    }

    fn save_display(&mut self) -> amethyst::Result<()> {
        fs::create_dir_all(self.project_dirs.config_dir())?;
        fs::write(
            self.display_config_path(),
            ron::ser::to_string_pretty(&self.display, PrettyConfig::default())?,
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    fn bindings_config_path(&self) -> PathBuf {
        bindings_config_path(&self.project_dirs)
    }

    fn display_config_path(&self) -> PathBuf {
        display_config_path(&self.project_dirs)
    }
}

fn bindings_config_path(project_dirs: &ProjectDirs) -> PathBuf {
    project_dirs.config_dir().join("bindings_config.ron")
}

fn display_config_path(project_dirs: &ProjectDirs) -> PathBuf {
    project_dirs.config_dir().join("display_config.ron")
}
