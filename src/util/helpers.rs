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
command.Workspaces = [" ", " "]

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
command.Custom = ["date +%H:%M", "date", { TimePassed = 60000 }, " %s"]

[[modules.right]]
command.Custom = ["iwgetid -r", "network", { TimePassed = 10000 }, "  %s"]

# CPU Module

# This module displays CPU usage. It takes two arguments:  the update time in milliseconds,
# and the formatting for the display (with "%s" as a placeholder for the value).

[[modules.right]]
command.Cpu = [5000, "󰍛 %s%"]

# Memory Module

# This module displays memory usage. It takes three arguments:
# the memory option (e.g., "PercUsed" to display the percentage of memory used),
# the update time in milliseconds, and the formatting for the display (with "%s" as a placeholder for the value).

[[modules.right]]
command.Memory = ["PercUsed", 5000, "󰍛 %s%"]

# Audio Module

# This module is designed to control and display the audio level. It takes two arguments:
# the display format (where "%s" is a placeholder for the value and "%c" stands for icons), and an array of icons.

[[modules.right]]
command.Audio = ["%c %s%", ["", "", "󰕾", ""]]

# Backlight Module

# This module is designed to show the level of screen backlight. It requires two arguments:
# the display format (where "%s" is a placeholder for the value and "%c" is a placeholder for icons), and an array of icons.

[[modules.right]]
command.Backlight = ["%c %s%", ["", "", "", "", "", "", "", "", ""]]

# Battery Module

# This module displays the battery status. It takes three arguments:
# the update time in milliseconds, the formatting for the display (with "%s" as a placeholder
# for the value and %c as a placeholder for icons), and an array of icons.

[[modules.right]]
command.Battery = [5000, "%c %s%", ["󰁺" ,"󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"]]
"#;

pub const CSS: &str = r#"
backlight {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-right: 10px;
    margin-left: 25px;
    margin-top: 10px;
}

battery {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-top: 10px;
}

audio {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-right: 25px;
    margin-top: 10px;
}

cpu {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-right: 25px;
    margin-top: 10px;
}

memory {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-right: 10px;
    margin-top: 10px;
}

workspaces {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-left: 35px;
    margin-top: 10px;
}

date {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-top: 10px;
}

network {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 16px;
    font-weight: bold;
    color: #ffffff;
    margin-right: 25px;
    margin-top: 10px;
}
"#;
