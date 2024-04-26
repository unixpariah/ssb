use super::{
    audio::{audio, AudioSettings},
    backlight::{backlight_details, BacklightSettings},
    battery::{battery_details, BatterySettings},
    cpu::{usage, CpuSettings},
    memory::{memory_usage, MemorySettings},
    persistant_workspaces::{persistant_workspaces, PersistantWorkspacesIcons},
    title::get_window_title,
    workspaces::{workspaces, WorkspacesIcons},
};
use crate::util::listeners::Trigger;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{error::Error, process::Command};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Cmd {
    Custom(CustomSettings),
    Workspaces(WorkspacesIcons),
    PersistantWorkspaces(PersistantWorkspacesIcons),
    Backlight(BacklightSettings),
    Memory(MemorySettings),
    Audio(AudioSettings),
    Cpu(CpuSettings),
    Battery(BatterySettings),
    WindowTitle,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CustomSettings {
    pub command: Box<str>,
    pub name: Box<str>,
    pub event: Trigger,
    pub formatting: String,
}

pub fn new_command(command: &str) -> Result<Box<str>, Box<dyn Error>> {
    let mut command_vec = command.split_whitespace().collect::<Vec<_>>();
    let output = String::from_utf8(
        Command::new(command_vec.remove(0))
            .args(command_vec)
            .output()?
            .stdout,
    )?
    .trim()
    .into();

    Ok(output)
}

pub fn get_command_output(command: &Cmd) -> Result<Box<str>, Box<dyn Error>> {
    Ok(match command {
        Cmd::Custom(settings) => match new_command(&settings.command) {
            Ok(output) => output,
            Err(e) => {
                warn!("Command '{}' failed, using default value", settings.command);
                Err(e)?
            }
        },
        Cmd::Workspaces(icons) => workspaces(icons),
        Cmd::PersistantWorkspaces(icons) => persistant_workspaces(&icons.0),
        Cmd::Memory(settings) => memory_usage(&settings.memory_opts)?,
        Cmd::Backlight(_) => backlight_details()?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu(_) => usage()?.split('.').next().ok_or("")?.into(),
        Cmd::Battery(_) => battery_details()?.into(),
        Cmd::Audio(_) => audio()?.into(),
        Cmd::WindowTitle => get_window_title().unwrap_or_default().into(),
    })
}
