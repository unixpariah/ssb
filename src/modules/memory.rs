use serde::{Deserialize, Serialize};
use sysinfo::System;

use std::sync::Arc;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct MemorySettings {
    pub memory_opts: MemoryOpts,
    pub interval: u64,
    pub formatting: Arc<str>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum MemoryOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

pub fn memory_usage(opt: &MemoryOpts) -> Box<str> {
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
    } as u8;

    output.to_string().into()
}
