use crate::config::CONFIG;
use cairo::{Context, TextExtents};

pub fn set_background_context(context: &Context) {
    let background = CONFIG.background;
    let font = &CONFIG.font;

    context.set_source_rgb(
        background[0] as f64 / 255.0,
        background[1] as f64 / 255.0,
        background[2] as f64 / 255.0,
    );
    let _ = context.paint();
    context.set_source_rgb(
        font.color[0] as f64 / 255.0,
        font.color[1] as f64 / 255.0,
        font.color[2] as f64 / 255.0,
    );
    context.select_font_face(
        &font.family,
        cairo::FontSlant::Normal,
        if font.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    context.set_font_size(font.size);
}

pub fn get_backlight_path() -> Result<std::path::PathBuf, Box<dyn crate::Error>> {
    let mut dirs = std::fs::read_dir("/sys/class/backlight")?;
    let backlight_path = dirs
        .find(|entry| {
            let entry = entry.as_ref().unwrap().path();
            if entry.join("brightness").exists() && entry.join("max_brightness").exists() {
                return true;
            }

            false
        })
        .ok_or("")??;

    Ok(backlight_path.path())
}

pub fn set_info_context(context: &Context, extents: TextExtents) {
    let background = CONFIG.background;
    let font = &CONFIG.font;

    context.set_source_rgb(
        background[0] as f64 / 255.0,
        background[1] as f64 / 255.0,
        background[2] as f64 / 255.0,
    );
    let _ = context.paint();

    context.move_to(extents.x_bearing().abs(), extents.y_bearing().abs());
    context.set_source_rgb(
        font.color[0] as f64 / 255.,
        font.color[1] as f64 / 255.,
        font.color[2] as f64 / 255.,
    );
    context.select_font_face(
        &font.family,
        cairo::FontSlant::Normal,
        if font.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    context.set_font_size(font.size);
}

pub const TOML: &str = r#"# Basic configurations

unkown = "N/A" # Default value for unknown commands
background = [20, 15, 33] # Background color as RGB value
topbar = true # true for bar at top of the screen, false for bar at bottom of the screen
height = 40 # Height of the bar

# Font settings

[font]
family = "JetBrainsMono Nerd Font"
size = 16.0
bold = true
color = [255, 255, 255]

# Modules

# Modules are individual components of the bar that display different information. 
# Each module has a `command` which determines what information it displays, 
# and `x` and `y` values which determine its position on the bar.

# Workspaces Module

# This module displays the active and inactive workspaces. It takes two arguments: 
# the icon for the active window and the icon for the inactive window.

[[modules]]
x = 35.0
y = 20.0
command.Workspaces = [" ", " "]

# Battery Module

# This module displays the battery status. It takes three arguments: 
# the battery option (e.g., "Capacity" to display the battery capacity), the update time in milliseconds, 
# and the formatting for the display (with "s%" as a placeholder for the value).

[[modules]]
x = 1390.0
y = 20.0
command.Battery = ["Capacity", 5000, " s%%"]

# Memory Module

# This module displays memory usage. It takes three arguments: 
# the memory option (e.g., "PercUsed" to display the percentage of memory used), 
# the update time in milliseconds, and the formatting for the display (with "s%" as a placeholder for the value).

[[modules]]
x = 1635.0
y = 20.0
command.Ram = ["PercUsed", 5000, "󰍛 s%%"]

# CPU Module

# This module displays CPU usage. It takes two arguments:  the update time in milliseconds, 
# and the formatting for the display (with "s%" as a placeholder for the value).

[[modules]]
x = 1700.0
y = 20.0
command.Cpu = [5000, " s%%"]

# Backlight Module

# This module displays the screen backlight level. It takes two arguments: the update time in milliseconds, 
# and the formatting for the display (with "s%" as a placeholder for the value).

[[modules]]
x = 1475.0
y = 20.0
command.Backlight = ["Perc", "󰖨 s%%"]

# Custom Module

# This module allows for custom commands. It takes three arguments: the command to execute, 
# the trigger event, and the formatting for the display (with "s%" as a placeholder for the value).

# Trigger Events

# Trigger events are conditions that cause a module to update. There are three types of trigger events:

# WorkspaceChanged

# This event is triggered when the active workspace changes. It doesn't take any arguments.

# FileChanged

# This event is triggered when a specified file changes. It takes one argument: the path to the file to monitor for changes.

# TimePassed

# This event is triggered at regular intervals. It takes one argument: the time in milliseconds between updates.

[[modules]]
x = 925.0
y = 20.0
command.Custom = ["date +%H:%M", { TimePassed = 60000 }, " s%"]

[[modules]]
x = 1775.0
y = 20.0
command.Custom = ["iwgetid -r", { TimePassed = 10000 }, "  s%"]

[[modules]]
x = 1540.0
y = 20.0
command.Custom = ["pamixer --get-volume", { TimePassed = 1000 }, " s%%"]
"#;
