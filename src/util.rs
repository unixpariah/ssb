use crate::{
    config::{BACKGROUND, FONT, UNKOWN},
    BacklightOpts, Cmd, Events, RamOpts, StatusData,
};
use cairo::Context;
use hyprland::shared::{HyprData, HyprDataActive, HyprDataVec};
use std::{error::Error, fs, process::Command, time::Instant};

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

pub fn get_ram(opt: RamOpts) -> Result<String, Box<dyn Error>> {
    let output = new_command("free", "-m")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let total = output[7].parse::<f64>()?;
    let used = output[8].parse::<f64>()?;

    Ok(match opt {
        RamOpts::PercUsed => (used / total) * 100.0,
        RamOpts::PercFree => ((total - used) / total) * 100.0,
        RamOpts::Used => used,
        RamOpts::Free => total - used,
    }
    .to_string())
}

pub fn get_backlight(opts: BacklightOpts) -> Result<String, Box<dyn Error>> {
    let brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/actual_brightness")?
        .trim()
        .parse::<f64>()?;

    let max_brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/max_brightness")?
        .trim()
        .parse::<f64>()?;

    match opts {
        BacklightOpts::Perc => Ok(((brightness / max_brightness) * 100.0).to_string()),
        BacklightOpts::Value => Ok(brightness.to_string()),
    }
}

pub fn get_cpu() -> Result<String, Box<dyn Error>> {
    let output = new_command("mpstat", "")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let idle = output.last().ok_or("not found")?.parse::<f64>()?;

    Ok((100.0 - idle).to_string())
}

pub fn get_current_workspace(
    active: &'static str,
    inactive: &'static str,
) -> Result<String, Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active().unwrap().id as usize - 1;
    let length = hyprland::data::Workspaces::get()?.to_vec().len();

    let o = (0..length)
        .map(|i| {
            if i == active_workspace {
                format!("{} ", active)
            } else {
                format!("{} ", inactive)
            }
        })
        .collect::<String>();

    Ok(o)
}

pub fn update_workspace_changed(info: &mut StatusData, events: &Events) {
    if let Ok(event) = events.active_window_change.1.try_recv() {
        if event {
            info.output = get_command_output(&info.command).unwrap_or(UNKOWN.to_string());
            if let Ok(tx) = events.active_window_change.0.try_borrow() {
                let _ = tx.send(false);
            }
        }
    }
}

pub fn update_time_passed(info: &mut StatusData, interval: u128) {
    if info.timestamp.elapsed().as_millis() >= interval {
        info.output = get_command_output(&info.command).unwrap_or(UNKOWN.to_string());
        info.timestamp = Instant::now();
    }
}

pub fn set_context_properties(context: &Context) {
    context.set_source_rgba(
        BACKGROUND[2] as f64 / 255.0,
        BACKGROUND[1] as f64 / 255.0,
        BACKGROUND[0] as f64 / 255.0,
        BACKGROUND[3] as f64 / 255.0,
    );
    let _ = context.paint();
    context.set_source_rgb(
        FONT.color[2] as f64 / 255.0,
        FONT.color[1] as f64 / 255.0,
        FONT.color[0] as f64 / 255.0,
    );
    context.select_font_face(
        FONT.family,
        cairo::FontSlant::Normal,
        if FONT.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    context.set_font_size(FONT.size);
}

pub fn get_command_output(command: &Cmd) -> Result<String, Box<dyn Error>> {
    Ok(match command {
        Cmd::Custom(command, args) => new_command(command, args)?,
        Cmd::Workspaces(active, inactive) => get_current_workspace(active, inactive)?,
        Cmd::Ram(opt) => get_ram(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Backlight(opt) => get_backlight(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu => get_cpu()?.split('.').next().ok_or("")?.into(),
    })
}
