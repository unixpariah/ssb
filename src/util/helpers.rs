use crate::config::{BACKGROUND, FONT};
use cairo::Context;

pub fn set_context_properties(context: &Context) {
    context.set_source_rgb(
        BACKGROUND[0] as f64 / 255.0,
        BACKGROUND[1] as f64 / 255.0,
        BACKGROUND[2] as f64 / 255.0,
    );
    let _ = context.paint();
    context.set_source_rgb(
        FONT.color[0] as f64 / 255.0,
        FONT.color[1] as f64 / 255.0,
        FONT.color[2] as f64 / 255.0,
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
