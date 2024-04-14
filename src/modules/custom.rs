use log::warn;
use serde::{Deserialize, Serialize};

use super::{
    audio::{audio, AudioSettings},
    backlight::{backlight_details, BacklightSettings},
    battery::{battery_details, BatterySettings},
    cpu::{usage, CpuSettings},
    memory::{memory_usage, MemorySettings},
    workspaces::{get_window_title, workspaces, WorkspacesIcons},
};
use crate::util::listeners::Trigger;
use std::{error::Error, process::Command};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Cmd {
    Custom(CustomSettings),
    Workspaces(WorkspacesIcons),
    Backlight(BacklightSettings),
    Memory(MemorySettings),
    Audio(AudioSettings),
    Cpu(CpuSettings),
    Battery(BatterySettings),
    WindowTitle,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CustomSettings {
    pub command: String,
    pub name: String,
    pub event: Trigger,
    pub formatting: String,
}

pub fn new_command(command: &str) -> Result<String, Box<dyn Error>> {
    let mut command_vec = command.split_whitespace().collect::<Vec<_>>();

    let output = String::from_utf8(
        Command::new(command_vec.remove(0))
            .args(command_vec)
            .output()?
            .stdout,
    )?
    .trim()
    .to_string();

    Ok(output)
}

pub fn get_command_output(command: &Cmd) -> Result<String, Box<dyn Error>> {
    Ok(match command {
        Cmd::Custom(settings) => match new_command(&settings.command) {
            Ok(output) => output,
            Err(e) => {
                warn!("Command '{}' failed, using default value", settings.command);
                Err(e)?
            }
        },
        Cmd::Workspaces(icons) => workspaces(icons),
        Cmd::Memory(settings) => memory_usage(&settings.memory_opts)?,
        Cmd::Backlight(_) => backlight_details()?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu(_) => usage()?.split('.').next().ok_or("")?.into(),
        Cmd::Battery(_) => battery_details()?,
        Cmd::Audio(_) => audio()?,
        Cmd::WindowTitle => get_window_title().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command() {
        assert!(new_command("echo test").is_ok());
        assert!(new_command("echo test").unwrap() == "test");
    }
}
