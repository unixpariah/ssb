use crate::{
    config::{self, Font},
    modules::custom::get_command_output,
    Cmd, ImgCache, ModuleData,
};
use cairo::{Context, ImageSurface, TextExtents};
use image::GenericImageView;
use rayon::prelude::*;

pub fn set_info_context(context: &Context, extents: TextExtents, config: &crate::config::Config) {
    let background = config.background;
    let font = &config.font;

    context.set_source_rgb(
        background[0] as f64 / 255.0,
        background[1] as f64 / 255.0,
        background[2] as f64 / 255.0,
    );
    _ = context.paint();

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

pub fn get_context(font: &Font) -> Context {
    let surface = ImageSurface::create(cairo::Format::Rgb30, 0, 0).unwrap();
    let context = cairo::Context::new(&surface).unwrap();

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
    context
}

pub fn update_config(
    information: &mut Vec<ModuleData>,
    config_changed: bool,
    config: &config::Config,
) -> bool {
    information
        .par_iter_mut()
        .map(|info| {
            if info.receiver.try_recv().is_ok() || info.output.is_empty() || config_changed {
                let output = get_command_output(&info.command).unwrap_or(config.unkown.to_string());

                if output != info.output || config_changed {
                    let format = info.format.replace("%s", &output);
                    let format = match &info.command {
                        Cmd::Battery(_, _, icons)
                        | Cmd::Backlight(_, icons)
                        | Cmd::Audio(_, icons)
                            if !icons.is_empty() =>
                        {
                            if let Ok(output) = output.parse::<usize>() {
                                let range_size = 100 / icons.len();
                                let icon =
                                    &icons[std::cmp::min(output / range_size, icons.len() - 1)];
                                format.replace("%c", icon)
                            } else {
                                format.replace("%c", "")
                            }
                        }
                        _ => format,
                    };
                    let context = get_context(&config.font);
                    let extents = match context.text_extents(&format) {
                        Ok(extents) => extents,
                        Err(_) => {
                            return false;
                        }
                    };

                    let (width, height) = info.cache.img.dimensions();
                    let width = if (extents.width() + extents.x_bearing().abs()) as u32 > width
                        || config_changed
                    {
                        extents.width() as u32
                    } else {
                        width
                    };

                    let height = if extents.height() as u32 > height || config_changed {
                        extents.height() as u32
                    } else {
                        height
                    };

                    let surface = match ImageSurface::create(
                        cairo::Format::Rgb30,
                        width as i32,
                        height as i32,
                    ) {
                        Ok(surface) => surface,
                        Err(_) => {
                            return false;
                        }
                    };
                    let context = match cairo::Context::new(&surface) {
                        Ok(context) => context,
                        Err(_) => {
                            return false;
                        }
                    };
                    set_info_context(&context, extents, config);

                    _ = context.show_text(&format);

                    let mut img = Vec::new();
                    _ = surface.write_to_png(&mut img);

                    if let Ok(img) = image::load_from_memory(&img) {
                        info.cache = ImgCache::new(img, false);
                    }

                    info.output = output;
                    return true;
                }
            };

            info.cache.unchanged = true;
            false
        })
        .reduce_with(|a, b| if b { b } else { a })
        .unwrap_or(false)
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

# Available for these compositors:
# - Hyprland

[[modules]]
x = 35.0
y = 20.0
command.Workspaces = [" ", " "]

# Battery Module

# This module displays the battery status. It takes three arguments: 
# the update time in milliseconds, the formatting for the display (with "%s" as a placeholder 
# for the value and %c as a placeholder for icons), and an array of icons.

[[modules]]
x = 1390.0
y = 20.0
command.Battery = [5000, "%c %s%", ["󰁺" ,"󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"]]

# Memory Module

# This module displays memory usage. It takes three arguments: 
# the memory option (e.g., "PercUsed" to display the percentage of memory used), 
# the update time in milliseconds, and the formatting for the display (with "%s" as a placeholder for the value).

[[modules]]
x = 1635.0
y = 20.0
command.Memory = ["PercUsed", 5000, "󰍛 %s%"]

# CPU Module

# This module displays CPU usage. It takes two arguments:  the update time in milliseconds, 
# and the formatting for the display (with "%s" as a placeholder for the value).

[[modules]]
x = 1700.0
y = 20.0
command.Cpu = [5000, " %s%"]

# Backlight Module

# This module is designed to show the level of screen backlight. It requires two arguments: 
# the display format (where "%s" is a placeholder for the value and "%c" is a placeholder for icons), and an array of icons.

[[modules]]
x = 1475.0
y = 20.0
command.Backlight = ["%c %s%", ["", "", "", "", "", "", "", "", ""]]

# Audio Module

# This module is designed to control and display the audio level. It takes two arguments: 
# the display format (where "%s" is a placeholder for the value and "%c" stands for icons), and an array of icons.

[[modules]]
x = 1540.0
y = 20.0
command.Audio = ["%c %s%", ["", "", "󰕾", ""]]

# Custom Module

# This module allows for custom commands. It takes three arguments: the command to execute, 
# the trigger event, and the formatting for the display (with "%s" as a placeholder for the value).

# Available trigger Events:

# WorkspaceChanged
# This event is triggered when the active workspace changes. It doesn't take any arguments.

# FileChanged
# This event is triggered when a specified file changes. It takes one argument: the path to the file to monitor for changes.

# TimePassed
# This event is triggered at regular intervals. It takes one argument: the time in milliseconds between updates.

# VolumeChanged
# This event is triggered when the volume changes. It doesn't take any arguments.

[[modules]]
x = 925.0
y = 20.0
command.Custom = ["date +%H:%M", { TimePassed = 60000 }, " %s"]

[[modules]]
x = 1775.0
y = 20.0
command.Custom = ["iwgetid -r", { TimePassed = 10000 }, "  %s"]
"#;
