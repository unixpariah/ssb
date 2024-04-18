use std::collections::HashMap;

use css_image::style::Style;
use image::{GenericImageView, ImageBuffer};
use lazy_static::lazy_static;

pub fn combine_images(images: &[&image::DynamicImage]) -> image::DynamicImage {
    let total_width = images.iter().map(|img| img.width()).sum();
    let max_height = images.iter().map(|img| img.height()).max().unwrap_or(0);
    let mut new_img = ImageBuffer::new(total_width, max_height);
    images.iter().fold(0, |acc, img| {
        let (width, height) = img.dimensions();
        (0..width).for_each(|x| {
            (0..height).for_each(|y| {
                let pixel = img.get_pixel(x, y);
                new_img.put_pixel(x + acc, y, pixel);
            });
        });
        acc + width
    });

    image::DynamicImage::ImageRgba8(new_img)
}

pub const TOML: &str = r#"
unkown = "N/A" # Default value for unknown commands
background = [20, 15, 33, 255] # Background color as RGB value
topbar = true # true for bar at top of the screen, false for bar at bottom of the screen
height = 40 # Height of the bar

# Font settings

# Modules

# Modules are individual components of the bar that display different information.
# Each module has a `command` which determines what information it displays,

# Workspaces Module

# This module displays the active and inactive workspaces. It takes two arguments:
# the icon for the active window and the icon for the inactive window.

# Available for these compositors:
# - Hyprland

[[modules.left]]
command.Workspaces = { active = " ", inactive = " " }

# Custom Module

# This module allows for custom commands. It takes four arguments: the command to execute,
# the trigger event, name for css selector, and the formatting for the display (with "%s" as a placeholder for the value).

# Available trigger Events:

# WorkspaceChanged
# This event is triggered when the active workspace changes. It doesn't take any arguments.

# FileChanged
# This event is triggered when a specified file is modified. It takes one argument: the path to the file

# TimePassed
# This event is triggered at regular intervals. It takes one argument: the time in milliseconds between updates.

# VolumeChanged
# This event is triggered when the volume changes. It doesn't take any arguments.

[[modules.center]]
command.Custom = { command = "date +%H:%M", name = "date", event = { TimePassed = 60000 }, formatting = " %s" }

[[modules.right]]
command.Custom = { command = "iwgetid -r", name = "network", event = { TimePassed = 10000 }, formatting = "  %s" }

# CPU Module

# This module displays CPU usage. It takes two arguments:  the update time in milliseconds,
# and the formatting for the display (with "%s" as a placeholder for the value).

[[modules.right]]
command.Cpu = { interval = 5000, formatting = "󰍛 %s%" }

# Memory Module

# This module displays memory usage. It takes three arguments:
# the memory option (e.g., "PercUsed" to display the percentage of memory used),
# the update time in milliseconds, and the formatting for the display (with "%s" as a placeholder for the value).

[[modules.right]]
command.Memory = { memory_opts = "PercUsed", interval = 5000, formatting = "󰍛 %s%" }

# Audio Module

# This module is designed to display the audio level. It takes two arguments:
# the display format (where "%s" is a placeholder for the value and "%c" stands for icons), and an array of icons.

[[modules.right]]
command.Audio = { formatting = "%c %s%", icons = ["", "", "󰕾", ""] }

# Backlight Module

# This module is designed to show the level of screen backlight. It requires two arguments:
# the display format (where "%s" is a placeholder for the value and "%c" is a placeholder for icons), and an array of icons.

[[modules.right]]
command.Backlight = { formatting = "%c %s%", icons = ["", "", "", "", "", "", "", "", ""] }

# Battery Module

# This module displays the battery status. It takes three arguments:
# the update time in milliseconds, the formatting for the display (with "%s" as a placeholder
# for the value and %c as a placeholder for icons), and an array of icons.

[[modules.right]]
command.Battery = { interval = 5000, formatting = "%c %s%", icons = ["󰁺" ,"󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"] }
"#;

pub const CSS_STRING: &str = r#"
* {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-top: 10px;
}

backlight {
    margin-left: 25px;
    margin-right: 10px;
}

audio {
    margin-right: 25px;
}

cpu {
    margin-right: 25px;
}

memory {
    margin-right: 10px;
}

workspaces {
    margin-left: 35px;
}

network {
    margin-right: 25px;
}

title {
    margin-right: 25px;
}
"#;

lazy_static! {
    pub static ref CSS: HashMap<String, Style> = css_image::parse(CSS_STRING).unwrap();
}
