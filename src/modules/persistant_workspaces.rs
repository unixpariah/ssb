use crate::get_style;

use super::workspaces::{hyprland, sway};
use css_image::style::Style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PersistantWorkspacesIcons(#[serde(default)] pub HashMap<String, String>);

pub fn persistant_workspaces(icons: &HashMap<String, String>) -> String {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    let (active_workspace, _) = match (hyprland_running, sway_running) {
        (true, _) => hyprland(),
        (_, true) => sway(),
        _ => unreachable!(), // Workspace listener wont work without sway or hyprland so no way to call this function anyways
    }
    .unwrap();

    (1..=10)
        .map(|i| {
            let index = i.to_string();
            match active_workspace == i {
                true => icons
                    .get("active")
                    .or_else(|| icons.get(&index))
                    .unwrap_or(&index)
                    .to_owned(),
                false => icons
                    .get(&index)
                    .or_else(|| icons.get("inactive"))
                    .unwrap_or(&index)
                    .to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn _render_persistant_workspaces(css: &HashMap<String, Style>, icons_str: &str) {
    let icons = icons_str.split_whitespace().collect::<Vec<&str>>();

    let mut width = 0;
    let icons = icons
        .iter()
        .enumerate()
        .map(|(i, icon)| {
            let a = get_style(css, &format!("persistant_workspaces#{i}"), icon);
            let img = image::load_from_memory(
                a.unwrap()
                    .get(&format!("persistant_workspaces#{i}"))
                    .unwrap(),
            )
            .unwrap()
            .to_rgba8();
            width += img.width();
            img
        })
        .collect::<Vec<_>>();

    let img_width = icons.iter().map(|icon| icon.width()).sum();
    let height = icons.iter().map(|icon| icon.height()).max().unwrap();
    let mut img = image::DynamicImage::new_rgba8(img_width, height);
    let mut width = 0;
    icons.iter().for_each(|icon| {
        image::imageops::replace(
            &mut img,
            icon,
            width as i64,
            (height - icon.height()) as i64,
        );
        width += icon.width();
    });
    img.save("persistant_workspaces.png").unwrap();
}
