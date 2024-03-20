use std::fs;

use crate::{util::listeners::Trigger, Cmd};
use inotify::{Inotify, WatchMask};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let inotify = Inotify::init().expect("Failed to setup inotify");

    let config_dir = dirs::config_dir().unwrap_or("".into());
    let config_path = config_dir.join("ssb/config.toml");

    inotify
        .watches()
        .add(&config_path, WatchMask::MODIFY)
        .expect("Failed to add watch");

    let file = fs::read_to_string(config_path).unwrap_or("".to_string());
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
    "Arial".to_string()
}

fn size() -> f64 {
    16.0
}

fn bold() -> bool {
    false
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
