use amethyst::{
    config::Config,
    input::{Bindings, StringBindings},
    window::{DisplayConfig, MonitorIdent},
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
        let project_dirs = ProjectDirs::from("", "Psychedelic Donkey", "hello-amethyst")
            .expect("Failed to get the project directory");
        fs::create_dir_all(project_dirs.config_dir())?;

        let bindings_config_path = bindings_config_path(&project_dirs);
        let bindings = Bindings::load_no_fallback(bindings_config_path.as_path()).or_else(
            move |_| -> amethyst::Result<Bindings<StringBindings>> {
                let bindings = Bindings::load_bytes(DEFAULT_BINDINGS_CONFIG_BYTES)?;
                fs::write(
                    bindings_config_path,
                    ron::ser::to_string_pretty(&bindings, PrettyConfig::default())?,
                )?;
                Ok(bindings)
            },
        )?;

        let display_config_path = display_config_path(&project_dirs);
        let display = DisplayConfig::load_no_fallback(display_config_path.as_path()).or_else(
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
