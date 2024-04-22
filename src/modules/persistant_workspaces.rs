use super::workspaces::{hyprland, sway};
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

    let length = 10;
    (1..=length)
        .map(|i| {
            let i = i.to_string();

            let string = match active_workspace.to_string() == i {
                true => "active",
                false => "inactive",
            };

            return icons
                .get(string)
                .or_else(|| icons.get(&i))
                .unwrap_or(&i)
                .to_owned();
        })
        .collect::<Vec<_>>()
        .join("")
}
