use log::warn;

use super::{
    audio::audio, backlight::backlight_details, battery::battery_details, cpu::usage,
    memory::memory_usage, workspaces::workspaces,
};
use crate::Cmd;
use std::{error::Error, process::Command};

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
        Cmd::Custom(command, _, _, _) => match new_command(command) {
            Ok(output) => output,
            Err(e) => {
                warn!("Command '{command}' failed, using default value");
                Err(e)?
            }
        },
        Cmd::Workspaces(workspace) => workspaces(workspace)?,
        Cmd::Memory(opt, _, _) => memory_usage(opt)?,
        Cmd::Backlight(_, _) => backlight_details()?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu(_, _) => usage()?.split('.').next().ok_or("")?.into(),
        Cmd::Battery(_, _, _) => battery_details()?,
        Cmd::Audio(_, _) => audio()?,
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
