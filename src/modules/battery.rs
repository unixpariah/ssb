use std::error::Error;

#[derive(Copy, Clone, Debug)]
pub enum BatteryOpts {
    Capacity,
    Status,
}

pub fn battery_details(opts: BatteryOpts) -> Result<String, Box<dyn Error>> {
    let mut dirs = std::fs::read_dir("/sys/class/power_supply")?;
    let path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("capacity").exists() && entry.join("status").exists() {
                return true;
            }

            false
        })
        .ok_or("")??;

    let capacity = std::fs::read_to_string(path.path().join("capacity"))?
        .trim()
        .to_string();
    let status = std::fs::read_to_string(path.path().join("status"))?
        .trim()
        .to_string();

    match opts {
        BatteryOpts::Capacity => Ok(capacity),
        BatteryOpts::Status => Ok(status),
    }
}
