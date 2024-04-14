use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BatterySettings {
    pub formatting: String,
    #[serde(default)]
    pub icons: Vec<String>,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BatteryOpts {
    Capacity,
    Status,
}

pub fn battery_details() -> Result<String, Box<dyn Error>> {
    let mut dirs = std::fs::read_dir("/sys/class/power_supply")?;
    let path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("capacity").exists() {
                return true;
            }

            false
        })
        .ok_or("")??;

    let capacity = std::fs::read_to_string(path.path().join("capacity"))?
        .trim()
        .to_string();

    Ok(capacity)
}
