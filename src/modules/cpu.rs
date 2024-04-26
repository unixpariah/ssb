use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CpuSettings {
    pub formatting: String,
    pub interval: u64,
}

pub fn usage() -> Result<Box<str>, Box<dyn Error>> {
    let mut system = System::new();
    system.refresh_cpu_usage();
    let usage = system.global_cpu_info().cpu_usage() as f64;

    Ok(usage.to_string().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage() {
        assert!(usage().is_ok());
    }
}
