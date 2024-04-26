use serde::{Deserialize, Serialize};
use sysinfo::System;

use std::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MemorySettings {
    pub memory_opts: MemoryOpts,
    pub interval: u64,
    pub formatting: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MemoryOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

pub fn memory_usage(opt: &MemoryOpts) -> Result<Box<str>, Box<dyn Error>> {
    let mut system = System::new();
    system.refresh_memory();
    let free = system.free_memory() as f64;
    let used = system.used_memory() as f64;
    let total = free + used;

    let output = match opt {
        MemoryOpts::PercUsed => (used / total) * 100.0,
        MemoryOpts::PercFree => (free / total) * 100.0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage() {
        let result = memory_usage(&MemoryOpts::Used);
        assert!(result.is_ok());
        assert!(result.unwrap().parse::<f64>().is_ok());

        let result = memory_usage(&MemoryOpts::Free);
        assert!(result.is_ok());
        assert!(result.unwrap().parse::<f64>().is_ok());

        let result = memory_usage(&MemoryOpts::PercUsed);
        assert!(result.is_ok());
        let result = result.unwrap().parse::<f64>();
        assert!(result.clone().is_ok());
        assert!(result.unwrap() <= 100.0);

        let result = memory_usage(&MemoryOpts::PercFree);
        assert!(result.is_ok());
        let result = result.unwrap().parse::<f64>();
        assert!(result.clone().is_ok());
        assert!(result.unwrap() <= 100.0);
    }
}
