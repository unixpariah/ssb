use crate::config::UNKOWN;
use std::{
    error::Error,
    fs,
    process::{Command, Output},
};

pub fn new_command(command: &str, args: &str) -> Vec<u8> {
    Command::new(command)
        .args(args.split_whitespace())
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

pub fn get_backlight() -> f64 {
    let brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/actual_brightness")
        .unwrap()
        .trim()
        .parse::<f64>()
        .unwrap();

    let max_brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/max_brightness")
        .unwrap()
        .trim()
        .parse::<f64>()
        .unwrap();

    (brightness / max_brightness) * 100.0
}

pub fn get_cpu() -> f64 {
    let output = new_command("mpstat", "");
    let output = String::from_utf8_lossy(&output);
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let idle = output.last().unwrap().parse::<f64>().unwrap();
    100.0 - idle
}

pub fn get_current_workspace() -> Result<String, Box<dyn Error>> {
    let workspaces = new_command("hyprctl", "workspaces -j");
    let workspaces = String::from_utf8(workspaces)?;

    let active_workspace = Command::new("hyprctl")
        .args(&["activeworkspace", "-j"])
        .output()?
        .stdout;
    let active_workspace = String::from_utf8(active_workspace)?;

    let active_workspace = serde_json::from_str::<serde_json::Value>(&active_workspace)?;
    let active_workspace = active_workspace.get("id").ok_or("")?.as_i64().ok_or("")? as usize - 1;

    let length = serde_json::from_str::<serde_json::Value>(&workspaces)?
        .as_array()
        .ok_or("")?
        .len();

    Ok((0..length)
        .map(|i| {
            if i == length - 1 && active_workspace >= length || i == active_workspace {
                "  "
            } else {
                "  "
            }
        })
        .collect::<String>())
}
