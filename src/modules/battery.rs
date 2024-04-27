use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct BatterySettings {
    pub formatting: Arc<str>,
    #[serde(default)]
    pub icons: Vec<Box<str>>,
    pub interval: u64,
}

#[derive(Deserialize)]
pub enum BatteryOpts {
    Capacity,
    Status,
}

pub fn battery_details() -> anyhow::Result<Box<str>> {
    let mut dirs = std::fs::read_dir("/sys/class/power_supply")?;
    let path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("capacity").exists() {
                return true;
            }

            false
        })
        .ok_or_else(|| anyhow::anyhow!("Battery not found"))??;

    let capacity = std::fs::read_to_string(path.path().join("capacity"))?
        .trim()
        .into();

    Ok(capacity)
}
