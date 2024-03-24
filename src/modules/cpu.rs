use sysinfo::System;

use std::error::Error;

pub fn usage() -> Result<String, Box<dyn Error>> {
    let mut system = System::new();
    system.refresh_cpu_usage();
    let usage = system.global_cpu_info().cpu_usage() as f64;

    Ok(usage.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage() {
        assert!(usage().is_ok());
    }
}
