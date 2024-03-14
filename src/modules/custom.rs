use super::{
    backlight::backlight_details, battery::battery_details, cpu_usage::cpu_usage, hyprland,
    memory::memory_usage,
};
use crate::Cmd;
use std::{error::Error, process::Command};

pub fn new_command(command: &str, args: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(
        Command::new(command)
            .args(args.split_whitespace())
            .output()?
            .stdout,
    )?
    .trim()
    .to_string())
}

pub fn get_command_output(command: &Cmd) -> Result<String, Box<dyn Error>> {
    Ok(match command {
        Cmd::Custom(command, args) => new_command(command, args)?,
        Cmd::Workspaces(active, inactive) => hyprland::workspaces(active, inactive)?,
        Cmd::Ram(opt) => memory_usage(*opt)?,
        Cmd::Backlight(opt) => backlight_details(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu => cpu_usage()?.split('.').next().ok_or("")?.into(),
        Cmd::Battery(opt) => battery_details(*opt)?,
    })
}
