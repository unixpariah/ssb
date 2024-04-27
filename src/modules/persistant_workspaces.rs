use crate::{get_style, CSS, MESSAGE};

use super::workspaces::{hyprland, sway};
use css_image::style::Style;
use image::DynamicImage;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct PersistantWorkspacesIcons(#[serde(default)] pub HashMap<Box<str>, Box<str>>);

pub fn persistant_workspaces(icons: &HashMap<Box<str>, Box<str>>) -> Box<str> {
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
            let index = i.to_string().into_boxed_str();
            let is_active = active_workspace == i;
            let string = if is_active { "active" } else { "inactive" };

            let mut icon = icons
                .get(string)
                .or_else(|| icons.get(&index))
                .unwrap_or(&index)
                .trim()
                .to_string();

            if !is_active && i < active_workspace {
                icon.push_str("\u{200B}\u{200B}");
            } else if is_active {
                icon.push('\u{200B}'); // Do not judge me
            }

            icon
        })
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

pub fn render(css: &[Style], icons: &str) -> DynamicImage {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    let (active_workspace, _) = match (hyprland_running, sway_running) {
        (true, _) => hyprland(),
        (_, true) => sway(),
        _ => unreachable!(), // Workspace listener wont work without sway or hyprland so no way to call this function anyways
    }
    .unwrap();

    let icons = icons.split_whitespace().collect::<Vec<&str>>();
    let icons = icons
        .iter()
        .enumerate()
        .map(|(i, icon)| {
            let mut name = "persistant_workspaces#".to_string();
            match active_workspace.checked_sub(1) {
                Some(active)
                    if active == i
                        && css
                            .iter()
                            .any(|a| a.selector == "persistant_workspaces#active") =>
                {
                    name.push_str("active");
                }
                _ if css
                    .iter()
                    .any(|a| a.selector == format!("persistant_workspaces#{i}")) =>
                {
                    name.push_str(&i.to_string());
                }
                _ if css
                    .iter()
                    .any(|a| a.selector == "persistant_workspaces#inactive") =>
                {
                    name.push_str("inactive")
                }
                _ => name.push_str(&i.to_string()),
            };

            let style = get_style(css, &name, icon).unwrap_or_else(|_| {
                let mut css = CSS
                    .iter()
                    .find(|a| a.selector == name.as_str())
                    .expect("Style declaration for module persistant_workspaces not found, using default style")
                    .to_owned();
                css.content.replace(icon.to_string().into_boxed_str());
                get_style(&vec![css], &name, icon).expect(MESSAGE)
            });
            let img_data = style.get(&*name).unwrap();
            image::load_from_memory(img_data).unwrap().to_rgba8()
        })
        .collect::<Vec<_>>();

    let mut persistant_workspaces = css
        .iter()
        .find(|a| a.selector == "persistant_workspaces")
        .unwrap_or_else(|| {
            warn!(
                "Style declaration for module persistant_workspaces not found, using default style"
            );
            CSS.iter()
                .find(|a| a.selector == "persistant_workspaces")
                .unwrap()
        })
        .clone();

    let letter_spacing = persistant_workspaces.font.letter_spacing;

    let img_height = icons.iter().map(|icon| icon.height()).max().unwrap_or(10) as i32;
    let img_width = icons.iter().map(|icon| icon.width() as i32).sum::<i32>()
        + letter_spacing as i32 * (icons.len() as i32 - 1);

    persistant_workspaces.width.replace(img_width);
    persistant_workspaces.height.replace(img_height - 10);

    let mut x = persistant_workspaces.margin[3] + persistant_workspaces.padding[3];
    let y = persistant_workspaces.margin[2] + persistant_workspaces.padding[2];

    let img = get_style(&vec![persistant_workspaces], "persistant_workspaces", "").unwrap();
    let mut img = image::load_from_memory(img.get("persistant_workspaces").unwrap())
        .unwrap()
        .to_rgba8();
    icons.iter().for_each(|icon| {
        image::imageops::overlay(&mut img, icon, x as i64, y as i64 - 10);
        x += icon.width() as i32 + letter_spacing as i32;
    });
    DynamicImage::from(img)
}
