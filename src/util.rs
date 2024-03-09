use crate::config::UNKOWN;
use std::process::{Command, Output};

pub fn new_command(command: &str, args: &str) -> Vec<u8> {
    Command::new(command)
        .arg(args)
        .output()
        .unwrap_or(Output {
            stdout: UNKOWN.into(),
            stderr: UNKOWN.into(),
            status: std::process::ExitStatus::default(),
        })
        .stdout
}

pub fn get_ram() -> f64 {
    let output = new_command("free", "-m");
    let output = String::from_utf8_lossy(&output);
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let total = output[7].parse::<f64>().unwrap();
    let used = output[8].parse::<f64>().unwrap();
    (used / total) * 100.0
}

pub fn get_wifi() -> String {
    let output = new_command("iwgetid", "-r");
    String::from_utf8(output).unwrap().trim().into()
}
