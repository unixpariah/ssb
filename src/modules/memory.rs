use serde::{Deserialize, Serialize};

use super::custom::new_command;
use std::error::Error;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MemoryOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

pub fn memory_usage(opt: MemoryOpts) -> Result<String, Box<dyn Error>> {
    let output = new_command("free -m")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let total = output[7].parse::<f64>()?;
    let used = output[8].parse::<f64>()?;

    let output = match opt {
        MemoryOpts::PercUsed => (used / total) * 100.0,
        MemoryOpts::PercFree => ((total - used) / total) * 100.0,
        MemoryOpts::Used => used,
        MemoryOpts::Free => total - used,
    }
    .to_string()
    .split('.')
    .next()
    .ok_or("")?
    .into();

    Ok(output)
}
