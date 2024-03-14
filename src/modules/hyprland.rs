use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use std::error::Error;

pub fn workspaces(active: &'static str, inactive: &'static str) -> Result<String, Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active().unwrap().id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    Ok((0..length)
        .map(|i| {
            if i == active_workspace - 1 || i == length - 1 && active_workspace > length {
                return format!("{} ", active);
            }
            format!("{} ", inactive)
        })
        .collect::<String>())
}
