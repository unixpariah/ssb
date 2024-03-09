use crate::Font;
use smithay_client_toolkit::shell::wlr_layer::Anchor;

pub const INTERVAL: u64 = 100; // In milliseconds

pub const UNKOWN: &[u8] = b"N/A";

// Background color               R   G   B   A
pub static BACKGROUND: [u8; 4] = [20, 15, 33, 255];

// Placement of status bar
// Options:
// - Anchor::TOP
// - Anchor::BOTTOM
pub static PLACEMENT: Anchor = Anchor::TOP;
pub static HEIGHT: i32 = 40; // Height of status bar in pixels
pub static FONT: Font = Font {
    font_family: "JetBrainsMono Nerd Font",
    font_size: 16.0,
    bolded: false,
    //      R    G    B
    color: [255, 255, 255],
};

pub static DATA: &[(Data, f64, f64, &str)] = &[
    // Command                         x      y     format
    (Data::Custom("date", "+%H:%M"), 925.0, 25.0, " $"),
    (Data::Ram, 1635.0, 25.0, "󰍛 $%"),
    (Data::Custom("iwgetid", "-r"), 1775.0, 25.0, "  $"),
    (Data::Backlight, 1475.0, 25.0, " $%"),
    (
        Data::Custom("pamixer", "--get-volume"),
        1540.0,
        25.0,
        " $%",
    ),
    (Data::Cpu, 1700.0, 25.0, " $%"),
    (Data::Workspaces, 35.0, 25.0, "$"),
];

pub enum Data {
    Custom(&'static str, &'static str),
    Ram,
    Backlight,
    Cpu,
    Workspaces,
}
