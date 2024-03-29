use std::error::Error;

pub fn get_backlight_path() -> Result<std::path::PathBuf, Box<dyn crate::Error>> {
    let mut dirs = std::fs::read_dir("/sys/class/backlight")?;
    let backlight_path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("brightness").exists() && entry.join("max_brightness").exists() {
                return true;
            }

            false
        })
        .ok_or("")??;

    Ok(backlight_path.path())
}

pub fn backlight_details() -> Result<String, Box<dyn Error>> {
    let path = get_backlight_path()?;

    let brightness = std::fs::read_to_string(path.join("brightness"))?
        .trim()
        .parse::<f32>()?;
    let max_brightness = std::fs::read_to_string(path.join("max_brightness"))?
        .trim()
        .parse::<f32>()?;

    Ok(((brightness / max_brightness) * 100.0).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlight_details() {
        match get_backlight_path() {
            Ok(_) => assert!(backlight_details().is_ok()),
            Err(_) => assert!(backlight_details().is_err()),
        }
    }
}
