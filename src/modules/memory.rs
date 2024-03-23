use serde::{Deserialize, Serialize};
use sysinfo::System;

use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub enum MemoryOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

pub fn memory_usage(opt: &MemoryOpts) -> Result<String, Box<dyn Error>> {
    let mut system = System::new();
    system.refresh_memory();
    let free = system.free_memory() as f64;
    let used = system.used_memory() as f64;
    let total = free + used;

    let output = match opt {
        MemoryOpts::PercUsed => (free / total) * 100.0,
        MemoryOpts::PercFree => (used / total) * 100.0,
        MemoryOpts::Used => used,
        MemoryOpts::Free => free,
    }
    .to_string()
    .split('.')
    .next()
    .ok_or("")?
    .into();

    Ok(output)
}
