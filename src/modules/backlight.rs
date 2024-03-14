use std::error::Error;

#[derive(Copy, Clone, Debug)]
pub enum BacklightOpts {
    Perc,
    Value,
}

pub fn backlight_details(opts: BacklightOpts) -> Result<String, Box<dyn Error>> {
    let mut dirs = std::fs::read_dir("/sys/class/backlight")?;
    let path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("brightness").exists() && entry.join("max_brightness").exists() {
                return true;
            }

            false
        })
        .ok_or("")??;

    let brightness = std::fs::read_to_string(path.path().join("brightness"))?
        .trim()
        .parse::<f32>()?;
    let max_brightness = std::fs::read_to_string(path.path().join("max_brightness"))?
        .trim()
        .parse::<f32>()?;

    match opts {
        BacklightOpts::Perc => Ok(((brightness / max_brightness) * 100.0).to_string()),
        BacklightOpts::Value => Ok(brightness.to_string()),
    }
}
