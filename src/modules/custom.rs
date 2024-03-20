use super::{
    backlight::backlight_details, battery::battery_details, cpu_usage::cpu_usage, hyprland,
    memory::memory_usage,
};
use crate::{Cmd, CONFIG};
use std::{error::Error, process::Command};

pub fn new_command(command: &str) -> Result<String, Box<dyn Error>> {
    let mut command = command.split_whitespace().collect::<Vec<_>>();

    let output = String::from_utf8(
        Command::new(command.remove(0))
            .args(command)
            .output()?
            .stdout,
    )?
    .trim()
    .to_string();

    if output.is_empty() {
        return Ok(CONFIG.unkown.to_string());
    }

    Ok(output)
}

pub fn get_command_output(command: &Cmd) -> Result<String, Box<dyn Error>> {
    Ok(match command {
        Cmd::Custom(command) => new_command(command)?,
        Cmd::Workspaces(workspace) => hyprland::workspaces(workspace)?,
        Cmd::Ram(opt) => memory_usage(*opt)?,
        Cmd::Backlight(opt) => backlight_details(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu => cpu_usage()?.split('.').next().ok_or("")?.into(),
        Cmd::Battery(opt) => battery_details(*opt)?,
    })
}
