use crate::{
    util::helpers::{CSS_STRING, TOML_STRING},
    Cmd,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};

pub fn get_css() -> Result<Box<str>, Box<dyn crate::Error>> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            warn!("Configuration directory not found, using default configuration");
            return Err("".into());
        }
    };
    let css_path = config_dir.join(format!("{}/style.css", env!("CARGO_PKG_NAME")));

    if !css_path.exists() {
        info!(
            "CSS file not found, generating new one at: {}",
            css_path.display()
        );
        fs::create_dir_all(css_path.parent().ok_or("")?)?;
        _ = fs::write(&css_path, CSS_STRING);
    }

    Ok(fs::read_to_string(&css_path)?.into())
}

pub fn get_config() -> Result<Arc<Config>, Box<dyn crate::Error>> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            warn!("Configuration directory not found, using default configuration");
            return Err("".into());
        }
    };
    let config_path = config_dir.join(format!("{}/config.toml", env!("CARGO_PKG_NAME")));
    if !config_path.exists() {
        info!(
            "Configuration file not found, generating new one at: {}",
            config_path.display()
        );
        fs::create_dir_all(config_path.parent().ok_or("")?)?;
        _ = fs::write(&config_path, TOML_STRING);
    }

    let config = toml::from_str::<Config>(fs::read_to_string(&config_path)?.trim())?;

    Ok(Arc::new(config))
}

#[derive(Deserialize, Serialize, Default)]
pub struct PositionedModules {
    pub left: Vec<Module>,
    pub center: Vec<Module>,
    pub right: Vec<Module>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "unkown")]
    pub unkown: Box<str>,
    #[serde(default = "background")]
    pub background: [f64; 4],
    #[serde(default = "topbar")]
    pub topbar: bool,
    #[serde(default = "layer")]
    pub layer: Box<str>,
    #[serde(default = "height")]
    pub height: i32,
    #[serde(default)]
    pub font: Font,
    #[serde(default)]
    pub modules: PositionedModules,
}

fn layer() -> Box<str> {
    "overlay".into()
}

fn unkown() -> Box<str> {
    "N/A".into()
}

fn background() -> [f64; 4] {
    [20., 15., 33., 1.]
}

fn topbar() -> bool {
    true
}

fn height() -> i32 {
    40
}

#[derive(Deserialize, Serialize)]
pub struct Module {
    pub command: Arc<Cmd>,
    #[serde(default = "pos")]
    pub x: f64,
    #[serde(default = "pos")]
    pub y: f64,
}

fn pos() -> f64 {
    0.0
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Font {
    #[serde(default = "family")]
    pub family: Box<str>,
    #[serde(default = "size")]
    pub size: f64,
    #[serde(default = "bold")]
    pub bold: bool,
    #[serde(default = "color")]
    pub color: [u8; 3],
}

fn family() -> Box<str> {
    "JetBrainsMono Nerd Font".into()
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
