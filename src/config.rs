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
