pub mod audio;
pub mod backlight;
pub mod battery;
pub mod cpu;
pub mod custom;
pub mod memory;
pub mod network;
pub mod persistant_workspaces;
pub mod title;
pub mod workspaces;

use self::{
    audio::AudioSettings,
    backlight::{get_backlight_path, BacklightSettings},
    battery::{battery_details, BatterySettings},
    cpu::CpuSettings,
    custom::{get_command_output, Cmd},
    memory::MemorySettings,
};
use crate::{
    config::Module,
    get_style,
    util::listeners::{Listeners, Trigger},
    HotConfig, Position, CSS, MESSAGE,
};
use css_image::style::Style;
use image::{ColorType, DynamicImage};
use log::warn;
use tokio::sync::broadcast;

pub struct ModuleData {
    pub output: Box<str>,
    pub command: Cmd,
    pub format: Box<str>,
    pub receiver: broadcast::Receiver<()>,
    pub cache: DynamicImage,
    pub position: Position,
}

impl ModuleData {
    pub fn new(listeners: &mut Listeners, module: &Module, position: &Position) -> Option<Self> {
        let (receiver, format) = match &module.command {
            Cmd::Workspaces(_) | Cmd::WindowTitle | Cmd::PersistantWorkspaces(_) => {
                (listeners.new_workspace_listener()?, "%s")
            }
            Cmd::Memory(MemorySettings { .. })
            | Cmd::Cpu(CpuSettings { .. })
            | Cmd::Battery(BatterySettings { .. })
                if matches!(&module.command, Cmd::Battery(_)) && battery_details().is_err() =>
            {
                warn!("Battery not found, deactivating module");
                return None;
            }
            Cmd::Memory(MemorySettings {
                interval,
                formatting,
                ..
            })
            | Cmd::Cpu(CpuSettings {
                interval,
                formatting,
                ..
            })
            | Cmd::Battery(BatterySettings {
                interval,
                formatting,
                ..
            }) => (listeners.new_time_listener(*interval), formatting.as_str()),
            Cmd::Backlight(settings) => {
                if let Ok(path) = get_backlight_path().map(|path| path.join("brightness")) {
                    (
                        listeners.new_file_listener(&path),
                        settings.formatting.as_str(),
                    )
                } else {
                    warn!("Backlight not found, deactivating module");
                    return None;
                }
            }
            Cmd::Audio(settings) => (
                listeners.new_volume_change_listener(),
                settings.formatting.as_str(),
            ),
            Cmd::Custom(settings) => {
                let trigger = match &settings.event {
                    Trigger::WorkspaceChanged => listeners.new_workspace_listener()?,
                    Trigger::TimePassed(interval) => listeners.new_time_listener(*interval),
                    Trigger::FileChange(path) => listeners.new_file_listener(path),
                    Trigger::VolumeChanged => listeners.new_volume_change_listener(),
                };
                (trigger, settings.formatting.as_str())
            }
        };

        Some(ModuleData {
            output: "".into(),
            command: module.command.clone(),
            format: format.into(),
            receiver,
            cache: DynamicImage::new(0, 0, ColorType::L8),
            position: position.clone(),
        })
    }

    pub fn render(&mut self, config_changed: bool, config: &HotConfig) {
        let output = get_command_output(&self.command).unwrap_or(config.config.unkown.clone());
        if output != self.output || config_changed {
            let format = self.format.replace("%s", &output);
            let format = match &self.command {
                Cmd::Battery(BatterySettings { icons, .. })
                | Cmd::Backlight(BacklightSettings { icons, .. })
                | Cmd::Audio(AudioSettings { icons, .. })
                    if !icons.is_empty() =>
                {
                    if let Ok(output) = output.parse::<usize>() {
                        let range_size = 100 / icons.len();
                        let icon = &icons[std::cmp::min(output / range_size, icons.len() - 1)];
                        format.replace("%c", icon)
                    } else {
                        format.replace("%c", "")
                    }
                }
                _ => format.replace("%c", ""),
            };

            let name = match &self.command {
                Cmd::PersistantWorkspaces(_) => "persistant_workspaces",
                Cmd::Workspaces(_) => "workspaces",
                Cmd::Memory(_) => "memory",
                Cmd::Cpu(_) => "cpu",
                Cmd::Battery(_) => "battery",
                Cmd::Backlight(_) => "backlight",
                Cmd::Audio(_) => "audio",
                Cmd::WindowTitle => "title",
                Cmd::Custom(custom) => &custom.name,
            };

            self.cache = match &self.command {
                Cmd::PersistantWorkspaces(_) => {
                    self.output = output;
                    persistant_workspaces::render(&config.css, &self.output)
                }
                _ => {
                    self.output = output;
                    generic_render(&config.css, name, &format)
                }
            };
        }
    }
}

fn generic_render(css: &[Style], name: &str, format: &str) -> DynamicImage {
    let img = get_style(css, name, format).unwrap_or_else(|_| {
        let mut css = CSS
            .iter()
            .find(|a| a.selector == name.into())
            .expect(MESSAGE)
            .to_owned();
        css.content = Some(format.into());
        css_image::render(css).expect(MESSAGE)
    });

    match img.get(name) {
        Some(img) => image::load_from_memory(img).unwrap(),
        None if img.get("*").is_some() => {
            let img = img.get("*").unwrap();
            image::load_from_memory(img).unwrap()
        }
        None => {
            warn!("Failed to parse {name} module css, using default style");
            let mut css = CSS
                .iter()
                .find(|a| a.selector == name.into())
                .expect(MESSAGE)
                .to_owned();
            css.content = Some(format.into());
            let css = css_image::render(css).expect(MESSAGE);
            image::load_from_memory(css.get(name).expect(MESSAGE)).unwrap()
        }
    }
}
