use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::util::helpers::get_backlight_path;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum BacklightOpts {
    Perc,
    Value,
}

pub fn backlight_details(opts: BacklightOpts) -> Result<String, Box<dyn Error>> {
    let path = get_backlight_path()?;

    let brightness = std::fs::read_to_string(path.join("brightness"))?
        .trim()
        .parse::<f32>()?;
    let max_brightness = std::fs::read_to_string(path.join("max_brightness"))?
        .trim()
        .parse::<f32>()?;

    match opts {
        BacklightOpts::Perc => Ok(((brightness / max_brightness) * 100.0).to_string()),
        BacklightOpts::Value => Ok(brightness.to_string()),
    }
}
