use crate::{util::helpers::TOML, Cmd};
use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;

lazy_static! {
    pub static ref CONFIG: Config = match get_config() {
        Ok(config) => config,
        Err(_) => {
            warn!("Error while parsing configuration file, using default configuration");
            toml::from_str(TOML).unwrap()
        }
    };
}

pub fn get_config() -> Result<Config, Box<dyn crate::Error>> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            warn!("Configuration directory not found, using default configuration");
            toml::from_str(TOML).unwrap()
        }
    };
    let config_path = config_dir.join("ssb/config.toml");

    if !config_path.exists() {
        info!(
            "Configuration file not found, generating new one at: {}",
            config_path.display()
        );
        fs::create_dir_all(config_path.parent().ok_or("")?)?;
        _ = fs::write(&config_path, TOML);
    }

    let file = fs::read_to_string(&config_path)?;

    Ok(toml::from_str::<Config>(file.trim())?)
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
