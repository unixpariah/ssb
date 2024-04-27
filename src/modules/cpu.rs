use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::System;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct CpuSettings {
    pub formatting: Arc<str>,
    pub interval: u64,
}

pub fn usage() -> Box<str> {
    let mut system = System::new();
    system.refresh_cpu_usage();
    let usage = system.global_cpu_info().cpu_usage() as u8;

    usage.to_string().into()
}
