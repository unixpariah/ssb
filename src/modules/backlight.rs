use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct BacklightSettings {
    pub formatting: Arc<str>,
    #[serde(default)]
    pub icons: Vec<Box<str>>,
}

pub fn get_backlight_path() -> anyhow::Result<std::path::PathBuf> {
    let mut dirs = std::fs::read_dir("/sys/class/backlight")?;
    let backlight_path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("brightness").exists() && entry.join("max_brightness").exists() {
                return true;
            }

            false
        })
        .ok_or_else(|| anyhow::anyhow!("Backlight path not found"))??;

    Ok(backlight_path.path())
}

pub fn backlight_details() -> anyhow::Result<Box<str>> {
    let path = get_backlight_path()?;

    let brightness = std::fs::read_to_string(path.join("brightness"))?
        .trim()
        .parse::<f32>()?;
    let max_brightness = std::fs::read_to_string(path.join("max_brightness"))?
        .trim()
        .parse::<f32>()?;

    let brightness = ((brightness / max_brightness) * 100.0) as u8;
    Ok((brightness).to_string().into())
}
