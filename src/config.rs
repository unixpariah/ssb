use std::fs;

use crate::{
    modules::{backlight::BacklightOpts, battery::BatteryOpts, memory::RamOpts},
    util::listeners::Trigger,
    Cmd,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let config_dir = dirs::config_dir().unwrap_or("".into());
    let config_path = config_dir.join("ssb/config.toml");

    let file = fs::read_to_string(&config_path).unwrap_or("".to_string());
    let config = toml::from_str::<Config>(file.trim());
    match config {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            let _ = fs::write(&config_path, toml::to_string(&config).unwrap());
            config
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let modules = vec![
            Module::workspace(),
            Module::battery(),
            Module::date(),
            Module::wifi(),
            Module::volume(),
            Module::memory(),
            Module::cpu(),
            Module::backlight(),
        ];

        Self {
            unkown: unkown(),
            background: background(),
            topbar: topbar(),
            height: height(),
            font: Font::default(),
            modules,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default = "unkown")]
    pub unkown: String,
    #[serde(default = "background")]
    pub background: [u8; 3],
    #[serde(default = "topbar")]
    pub topbar: bool,
    #[serde(default = "height")]
    pub height: i32,
    #[serde(default)]
    pub font: Font,
    pub modules: Vec<Module>,
}

fn unkown() -> String {
    "N/A".to_string()
}

fn background() -> [u8; 3] {
    [20, 15, 33]
}

fn topbar() -> bool {
    true
}

fn height() -> i32 {
    40
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Module {
    pub command: Cmd,
    #[serde(default = "pos")]
    pub x: f64,
    #[serde(default = "pos")]
    pub y: f64,
    #[serde(default = "format")]
    pub format: String,
    #[serde(default = "trigger")]
    pub trigger: Trigger,
}

impl Module {
    pub fn workspace() -> Self {
        Self {
            command: Cmd::Workspaces([" ".to_string(), " ".to_string()]),
            x: 35.0,
            y: 20.0,
            format: "s%".to_string(),
            trigger: Trigger::WorkspaceChanged,
        }
    }

    pub fn battery() -> Self {
        Self {
            command: Cmd::Battery(BatteryOpts::Capacity),
            x: 1390.0,
            y: 20.0,
            format: " s%%".to_string(),
            trigger: Trigger::TimePassed(5000),
        }
    }

    pub fn date() -> Self {
        Self {
            command: Cmd::Custom("date +%H:%M".to_string()),
            x: 925.0,
            y: 20.0,
            format: " s%".to_string(),
            trigger: Trigger::TimePassed(60000),
        }
    }

    pub fn wifi() -> Self {
        Self {
            command: Cmd::Custom("iwgetid -r".to_string()),
            x: 1775.0,
            y: 20.0,
            format: "  s%".to_string(),
            trigger: Trigger::TimePassed(60000),
        }
    }

    pub fn volume() -> Self {
        Self {
            command: Cmd::Custom("pamixer --get-volume".to_string()),
            x: 1540.0,
            y: 20.0,
            format: " s%%".to_string(),
            trigger: Trigger::TimePassed(1000),
        }
    }

    pub fn memory() -> Self {
        Self {
            command: Cmd::Ram(RamOpts::PercUsed),
            x: 1635.0,
            y: 20.0,
            format: "󰍛 s%%".to_string(),
            trigger: Trigger::TimePassed(5000),
        }
    }

    pub fn cpu() -> Self {
        Self {
            command: Cmd::Cpu,
            x: 1700.0,
            y: 20.0,
            format: " s%%".to_string(),
            trigger: Trigger::TimePassed(5000),
        }
    }

    pub fn backlight() -> Self {
        Self {
            command: Cmd::Backlight(BacklightOpts::Perc),
            x: 1475.0,
            y: 20.0,
            format: "󰖨 s%%".to_string(),
            trigger: Trigger::TimePassed(5000),
        }
    }
}

fn pos() -> f64 {
    0.0
}

fn trigger() -> Trigger {
    Trigger::TimePassed(5000)
}

fn format() -> String {
    "s%".to_string()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Font {
    #[serde(default = "family")]
    pub family: String,
    #[serde(default = "size")]
    pub size: f64,
    #[serde(default = "bold")]
    pub bold: bool,
    #[serde(default = "color")]
    pub color: [u8; 3],
}

fn family() -> String {
    "JetBrainsMono Nerd Font".to_string()
}

fn size() -> f64 {
    16.0
}

fn bold() -> bool {
    true
}

fn color() -> [u8; 3] {
    [255, 255, 255]
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: family(),
            size: size(),
            bold: bold(),
            color: color(),
        }
    }
}
