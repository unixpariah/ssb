use crate::{BacklightOpts, Cmd, CpuOpts, Font, RamOpts};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

// Value to display when data is not available
pub const UNKOWN: &str = "N/A";

// Background color               R   G   B   A
pub static BACKGROUND: [u8; 4] = [20, 15, 33, 255];

// Placement of status bar
// Options:
// - Anchor::TOP
// - Anchor::BOTTOM
pub static PLACEMENT: Anchor = Anchor::TOP;
pub static HEIGHT: i32 = 40; // Height of status bar in pixels
pub static FONT: Font = Font {
    family: "JetBrainsMono Nerd Font",
    size: 16.0,
    bold: true,
    //      R    G    B
    color: [255, 255, 255],
};

/*
 *  Function          Description                                                Argument                                                                 Example
 *
 *  Cmd::Custom       This function allows you to run any custom command         It takes two arguments: the command to run and its arguments             ("pamixer", "--get-volume")
 *                    that you specify.                                          These are typically passed as strings.
 *
 *  Cmd::Workspaces   This function is used to display the status of all         It takes two arguments: icons for active and inactive windows            (" ", " ")
 *                    workspaces.                                                These are typically passed as strings.
 *
 *  Cmd::Backlight    This function provides information about the backlight     It takes one argument: an enum representing the type of data to display  (BacklightOpts::Perc)
 *                    of your display.                                           This could be the percentage of brightness, the actual brightness value,
 *                                                                               or any other relevant information.
 *
 *  Cmd::Ram          This function gives information about the system's RAM.    It takes one argument: an enum representing the type of data to display   (RamOpts::PercUsed)
 *                                                                               This could be the percentage of RAM used, the actual amount of RAM used,
 *                                                                               or any other relevant information.
 *
 *  Cmd::Cpu          This function provides information about the CPU.          It takes one argument: an enum representing the type of data to display   (CpuOpts::Perc)
 *                                                                               This could be the percentage of CPU used, the actual CPU speed,
 *                                                                               or any other relevant information.
 */

#[rustfmt::skip]
pub static DATA: [(Cmd, f64, f64, &str, usize); 7] = [
    // Command                                x        y      format    interval(ms)
    (Cmd::Custom("pamixer", "--get-volume"),  1540.0,  25.0,  " s%%",  50     ),
    (Cmd::Custom("date", "+%H:%M"),           925.0,   25.0,  " s%",   60000  ),
    (Cmd::Custom("iwgetid", "-r"),            1775.0,  25.0,  "  s%",  60000  ),
    (Cmd::Workspaces(" ", " "),             35.0,    25.0,  "s%",     50     ),
    (Cmd::Backlight(BacklightOpts::Perc),     1475.0,  25.0,  "󰖨 s%%",  50     ),
    (Cmd::Ram(RamOpts::PercUsed),             1635.0,  25.0,  "󰍛 s%%",  5000   ),
    (Cmd::Cpu(CpuOpts::Perc),                 1700.0,  25.0,  " s%%",  5000   ),
];
