use crate::{
    modules::{backlight::BacklightOpts, battery::BatteryOpts, memory::RamOpts},
    util::{helpers::TOML, listeners::Trigger},
    Cmd,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;

lazy_static! {
    pub static ref CONFIG: Config = match get_config() {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Error while parsing configuration file, using default one");
            Config::default()
        }
    };
}

pub fn get_config() -> Result<Config, Box<dyn crate::Error>> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Configuration directory not found");
            return Ok(Config::default());
        }
    };
    let config_path = config_dir.join("ssb/config.toml");

    if !config_path.exists() {
        println!(
            "Configuration file not found, generating a new one at: {:?}",
            config_path
        );
        let _ = fs::write(&config_path, TOML);
    }

    let file = fs::read_to_string(&config_path)?;

    Ok(toml::from_str::<Config>(file.trim())?)
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
}

impl Module {
    pub fn workspace() -> Self {
        Self {
            command: Cmd::Workspaces([" ".to_string(), " ".to_string()]),
            x: 35.0,
            y: 20.0,
        }
    }

    pub fn battery() -> Self {
        Self {
            command: Cmd::Battery(BatteryOpts::Capacity, 5000, " s%%".to_string()),
            x: 1390.0,
            y: 20.0,
        }
    }

    pub fn date() -> Self {
        Self {
            command: Cmd::Custom(
                "date +%H:%M".to_string(),
                Trigger::TimePassed(60000),
                " s%".to_string(),
            ),
            x: 925.0,
            y: 20.0,
        }
    }

    pub fn wifi() -> Self {
        Self {
            command: Cmd::Custom(
                "iwgetid -r".to_string(),
                Trigger::TimePassed(10000),
                "  s%".to_string(),
            ),
            x: 1775.0,
            y: 20.0,
        }
    }

    pub fn volume() -> Self {
        Self {
            command: Cmd::Custom(
                "pamixer --get-volume".to_string(),
                Trigger::TimePassed(1000),
                " s%%".to_string(),
            ),
            x: 1540.0,
            y: 20.0,
        }
    }

    pub fn memory() -> Self {
        Self {
            command: Cmd::Ram(RamOpts::PercUsed, 5000, "󰍛 s%%".to_string()),
            x: 1635.0,
            y: 20.0,
        }
    }

    pub fn cpu() -> Self {
        Self {
            command: Cmd::Cpu(5000, " s%%".to_string()),
            x: 1700.0,
            y: 20.0,
        }
    }

    pub fn backlight() -> Self {
        Self {
            command: Cmd::Backlight(BacklightOpts::Perc, "󰖨 s%%".to_string()),
            x: 1475.0,
            y: 20.0,
        }
    }
}

fn pos() -> f64 {
    0.0
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
