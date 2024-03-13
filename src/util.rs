use crate::{
    config::{BACKGROUND, FONT},
    Cmd,
};
use cairo::Context;
use hyprland::{
    event_listener::EventListener,
    shared::{HyprData, HyprDataActive},
};
use std::{
    cell::RefCell, error::Error, fs, process::Command, rc::Rc, sync::mpsc::Receiver, thread,
    time::Duration,
};

#[derive(Copy, Clone)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(&'static str),
}

#[derive(Copy, Clone)]
pub enum BacklightOpts {
    Perc,
    Value,
}

#[derive(Copy, Clone)]
pub enum RamOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

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

    let output = match opt {
        RamOpts::PercUsed => (used / total) * 100.0,
        RamOpts::PercFree => ((total - used) / total) * 100.0,
        RamOpts::Used => used,
        RamOpts::Free => total - used,
    }
    .to_string()
    .split('.')
    .next()
    .ok_or("")?
    .into();

    Ok(output)
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

    let output = (100.0 - idle)
        .to_string()
        .split('.')
        .next()
        .ok_or("")?
        .into();

    Ok(output)
}

pub fn get_current_workspace(
    active: &'static str,
    inactive: &'static str,
) -> Result<String, Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active().unwrap().id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    Ok((0..length)
        .map(|i| {
            if i == active_workspace - 1 || i == length - 1 && active_workspace > length {
                return format!("{} ", active);
            }
            format!("{} ", inactive)
        })
        .collect::<String>())
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
        Cmd::Ram(opt) => get_ram(*opt)?,
        Cmd::Backlight(opt) => get_backlight(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Cpu => get_cpu()?.split('.').next().ok_or("")?.into(),
    })
}

pub fn create_workspace_listener() -> Receiver<bool> {
    let mut listener = EventListener::new();
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = Rc::new(RefCell::new(tx));

    {
        let tx = Rc::clone(&tx);
        listener.add_workspace_destroy_handler(move |_| {
            let _ = tx.borrow().send(true);
        });
    }

    {
        let tx = Rc::clone(&tx);
        listener.add_workspace_change_handler(move |_| {
            let _ = tx.borrow().send(true);
        });
    }

    {
        let tx = Rc::clone(&tx);
        listener.add_active_monitor_change_handler(move |_| {
            let _ = tx.borrow().send(true);
        });
    }

    thread::spawn(move || {
        listener.start_listener().expect("Failed to start listener");
    });

    rx
}

pub fn create_time_passed_listener(interval: u64) -> Receiver<bool> {
    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(interval));
        let _ = tx.send(true);
    });

    rx
}

pub fn create_file_change_listener(path: &'static str) -> Receiver<bool> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut hotwatch = hotwatch::Hotwatch::new_with_custom_delay(Duration::ZERO)
        .expect("Failed to create hotwatch");
    hotwatch
        .watch(path, move |event: hotwatch::Event| {
            if let hotwatch::EventKind::Modify(_) = event.kind {
                let _ = tx.send(true);
            }
        })
        .unwrap();

    rx
}
