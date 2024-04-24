use crate::get_style;

use super::workspaces::{hyprland, sway};
use css_image::style::Style;
use image::DynamicImage;
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

pub fn render(css: &HashMap<String, Style>, icons_str: &str) -> DynamicImage {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    let (active_workspace, _) = match (hyprland_running, sway_running) {
        (true, _) => hyprland(),
        (_, true) => sway(),
        _ => unreachable!(), // Workspace listener wont work without sway or hyprland so no way to call this function anyways
    }
    .unwrap();

    let icons = icons_str.split_whitespace().collect::<Vec<&str>>();
    let icons = icons
        .iter()
        .enumerate()
        .map(|(i, icon)| {
            let format = match active_workspace.checked_sub(1) {
                Some(active) if active == i && css.contains_key("persistant_workspaces#active") => {
                    "persistant_workspaces#active".to_string()
                }
                _ if css.contains_key(&format!("persistant_workspaces#{i}")) => {
                    format!("persistant_workspaces#{i}")
                }
                _ if css.contains_key("persistant_workspaces#inactive") => {
                    "persistant_workspaces#inactive".to_string()
                }
                _ => format!("persistant_workspaces#{i}"),
            };

            let style = get_style(css, &format, icon).unwrap();
            let img_data = style.get(&format).unwrap();
            image::load_from_memory(img_data).unwrap().to_rgba8()
        })
        .collect::<Vec<_>>();

    let mut persistant_workspaces = css.get("persistant_workspaces").unwrap().clone();
    let letter_spacing = persistant_workspaces.font.letter_spacing;
    let img_height = icons.iter().map(|icon| icon.height()).max().unwrap() as i32 - 10;
    let img_width = icons.iter().map(|icon| icon.width() as i32).sum::<i32>()
        + letter_spacing as i32 * (icons.len() as i32 - 1);
    persistant_workspaces.width = Some(img_width);
    persistant_workspaces.height = Some(img_height);
    let mut x = persistant_workspaces.margin[3] + persistant_workspaces.padding[3];
    let y = persistant_workspaces.margin[2] + persistant_workspaces.padding[2];
    let css: HashMap<String, Style> =
        [(String::from("persistant_workspaces"), persistant_workspaces)].into();
    let img = get_style(&css, "persistant_workspaces", "").unwrap();
    let mut img = image::load_from_memory(img.get("persistant_workspaces").unwrap())
        .unwrap()
        .to_rgba8();
    icons.iter().for_each(|icon| {
        image::imageops::overlay(
            &mut img,
            icon,
            x as i64,
            (img_height + 10 - icon.height() as i32) as i64 - y as i64,
        );
        x += icon.width() as i32 + letter_spacing as i32;
    });
    DynamicImage::from(img)
}
