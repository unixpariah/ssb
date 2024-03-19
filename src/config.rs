use std::fs;

use crate::{
    modules::{backlight::BacklightOpts, battery::BatteryOpts, memory::RamOpts},
    util::listeners::Trigger,
    Cmd,
};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

fn get_config() -> Config {
    let config_dir = dirs::config_dir().unwrap_or("".into());
    let file = fs::read_to_string(format!("{}/ssb/config.toml", config_dir.display()))
        .unwrap_or("".to_string());
    toml::from_str(file.trim()).unwrap_or_default()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            unkown: unkown(),
            background: background(),
            topbar: topbar(),
            height: height(),
            font: Font::default(),
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "unkown")]
    pub unkown: String,
    #[serde(default = "background")]
    pub background: [u8; 3],
    #[serde(default = "topbar")]
    pub topbar: bool,
    #[serde(default = "height")]
    pub height: i32,
    #[serde(default)]
    pub font: Font,
}

#[derive(Deserialize)]
pub struct Font {
    pub family: String,
    pub size: f64,
    pub bold: bool,
    pub color: [u8; 3],
}

fn unkown() -> String {
    "N/A".to_string()
}

fn background() -> [u8; 3] {
    [20, 15, 33]
}

fn topbar() -> bool {
    true
}

fn height() -> i32 {
    40
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: "JetBrainsMono Nerd Font".to_string(),
            size: 16.0,
            bold: false,
            color: [255, 255, 255],
        }
    }
}

const BACKLIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";

#[rustfmt::skip]
pub const COMMAND_CONFIGS: &[(Cmd, f64, f64, &str, Trigger)] = &[
    // Command                                x        y      format    Trigger
    (Cmd::Battery(BatteryOpts::Capacity),     1390.0,  20.0,  " s%%",  Trigger::TimePassed(1010)            ),
    (Cmd::Custom("pamixer", "--get-volume"),  1540.0,  20.0,  " s%%",  Trigger::TimePassed(1000)            ), 
    (Cmd::Custom("date", "+%H:%M"),           925.0,   20.0,  " s%",   Trigger::TimePassed(60000)           ),
    (Cmd::Custom("iwgetid", "-r"),            1775.0,  20.0,  "  s%",  Trigger::TimePassed(60000)           ),
    (Cmd::Backlight(BacklightOpts::Perc),     1475.0,  20.0,  "󰖨 s%%",  Trigger::FileChange(BACKLIGHT_PATH)  ),
    (Cmd::Workspaces(" ", " "),             35.0,    20.0,  "s%",     Trigger::WorkspaceChanged            ),
    (Cmd::Ram(RamOpts::PercUsed),             1635.0,  20.0,  "󰍛 s%%",  Trigger::TimePassed(5000)            ),
    (Cmd::Cpu,                                1700.0,  20.0,  " s%%",  Trigger::TimePassed(5000)            ),
];
