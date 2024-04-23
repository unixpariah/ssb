pub mod audio;
pub mod backlight;
pub mod battery;
pub mod cpu;
pub mod custom;
pub mod memory;
pub mod persistant_workspaces;
pub mod title;
pub mod workspaces;

use self::{
    audio::AudioSettings,
    backlight::BacklightSettings,
    battery::BatterySettings,
    custom::{get_command_output, Cmd},
};
use crate::{get_style, HotConfig, Position, CSS, MESSAGE};
use css_image::style::Style;
use image::DynamicImage;
use log::warn;
use std::collections::HashMap;
use tokio::sync::broadcast;

pub struct ModuleData {
    pub output: String,
    pub command: Cmd,
    pub format: String,
    pub receiver: broadcast::Receiver<()>,
    pub cache: DynamicImage,
    pub position: Position,
}

impl ModuleData {
    pub fn render(&mut self, config_changed: bool, config: &HotConfig) {
        let output = get_command_output(&self.command).unwrap_or(config.config.unkown.to_string());
        //if let Cmd::PersistantWorkspaces(_) = &self.command {
        //render_persistant_workspaces(&config.css, &output);
        //}

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

            let img = get_style(&config.css, name, &format).unwrap_or_else(|_| {
                let mut css = CSS.get(name).expect(MESSAGE).to_owned();
                css.content = Some(format.to_string());
                let css: HashMap<String, Style> =
                    [(name.to_string(), css)].iter().cloned().collect();
                css_image::render(css).expect(MESSAGE)
            });

            self.cache = match img.get(name) {
                Some(img) => image::load_from_memory(img).unwrap(),
                None if img.get("*").is_some() => {
                    let img = img.get("*").unwrap();
                    image::load_from_memory(img).unwrap()
                }
                None => {
                    warn!("Failed to parse {name} module css, using default style");
                    let mut css = CSS.get(name).expect(MESSAGE).to_owned();
                    css.content = Some(format.to_string());
                    let css: HashMap<String, Style> =
                        [(name.to_string(), css)].iter().cloned().collect();
                    let css = css_image::render(css).expect(MESSAGE);
                    image::load_from_memory(css.get(name).expect(MESSAGE)).unwrap()
                }
            };

            self.output = output;
        }
    }
}
