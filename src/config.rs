use crate::{Data, Font};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

// Value to display when data is not available
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

#[rustfmt::skip]
pub static DATA: &[(Data, f64, f64, &str, usize)] = &[
    // Command                                x       y     format  interval(ms)
    (Data::Custom("pamixer", "--get-volume"), 1540.0, 25.0, " $%", 1000  ),
    (Data::Custom("date", "+%H:%M"),          925.0,  25.0, " $",  60000 ),
    (Data::Custom("iwgetid", "-r"),           1775.0, 25.0, "  $", 1000  ),
    (Data::Workspaces,                        35.0,   25.0, "$",    0     ),
    (Data::Backlight,                         1475.0, 25.0, " $%", 1000  ),
    (Data::Ram,                               1635.0, 25.0, "󰍛 $%", 1000  ),
    (Data::Cpu,                               1700.0, 25.0, " $%", 1000  ),
];
