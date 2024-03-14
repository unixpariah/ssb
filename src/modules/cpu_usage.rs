use super::custom::new_command;
use std::error::Error;

pub fn cpu_usage() -> Result<String, Box<dyn Error>> {
    let output = new_command("mpstat", "")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let idle = output.last().ok_or("not found")?.parse::<f64>()?;

    let output = (100.0 - idle)
        .to_string()
        .split('.')
        .next()
        .ok_or("")?
        .into();

    Ok(output)
}
