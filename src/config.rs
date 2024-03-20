use std::fs;

use crate::{util::listeners::Trigger, Cmd};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let config_dir = dirs::config_dir().unwrap_or("".into());
    let config_path = config_dir.join("ssb/config.toml");

    let file = fs::read_to_string(config_path.clone()).unwrap_or("".to_string());
    toml::from_str::<Config>(file.trim()).unwrap_or_default()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            unkown: unkown(),
            background: background(),
            topbar: topbar(),
            height: height(),
            font: Font::default(),
            modules: Vec::new(),
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Module {
    pub command: Cmd,
    pub x: f64,
    pub y: f64,
    pub format: String,
    pub trigger: Trigger,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Font {
    pub family: String,
    pub size: f64,
    pub bold: bool,
    pub color: [u8; 3],
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

impl Default for Font {
    fn default() -> Self {
        Self {
            family: "JetBrainsMono Nerd Font".to_string(),
            size: 16.0,
            bold: false,
            color: [255, 255, 255],
        }
    }
}
