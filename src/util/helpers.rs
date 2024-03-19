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
