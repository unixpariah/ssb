use crate::{
    util::helpers::{CSS, TOML},
    Cmd,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;

pub fn get_config() -> Result<(Config, String), Box<dyn crate::Error>> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            warn!("Configuration directory not found, using default configuration");
            return Err("".into());
        }
    };
    let config_path = config_dir.join(format!("{}/config.toml", env!("CARGO_PKG_NAME")));
    let css_path = config_dir.join(format!("{}/style.css", env!("CARGO_PKG_NAME")));

    if !config_path.exists() {
        info!(
            "Configuration file not found, generating new one at: {}",
            config_path.display()
        );
        fs::create_dir_all(config_path.parent().ok_or("")?)?;
        _ = fs::write(&config_path, TOML);
    }

    if !css_path.exists() {
        info!(
            "CSS file not found, generating new one at: {}",
            css_path.display()
        );
        fs::create_dir_all(css_path.parent().ok_or("")?)?;
        _ = fs::write(&css_path, CSS);
    }

    let config = toml::from_str::<Config>(&fs::read_to_string(&config_path)?.trim())?;
    let css = fs::read_to_string(&css_path)?;

    Ok((config, css))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    #[serde(default)]
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

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Clone, Deserialize, Serialize, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        _ = get_config();
    }
}
