use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WorkspacesIcons {
    pub active: String,
    pub inactive: String,
}

pub fn workspaces(workspace: &WorkspacesIcons) -> String {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    let (active_workspace, length) = match (hyprland_running, sway_running) {
        (true, _) => hyprland(),
        (_, true) => sway(),
        _ => unreachable!(), // Workspace listener wont work without sway or hyprland so no way to call this function anyways
    }
    .unwrap();

    (0..length).fold(String::new(), |mut workspace_state, i| {
        let workspace = if i == active_workspace - 1 || i == length - 1 && active_workspace > length
        {
            &workspace.active
        } else {
            &workspace.inactive
        };
        workspace_state.push_str(workspace);
        workspace_state.push(' ');

        workspace_state
    })
}

pub fn hyprland() -> Result<(usize, usize), Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active()?.id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    Ok((active_workspace, length))
}

pub fn sway() -> Result<(usize, usize), Box<dyn Error>> {
    let workspaces = swayipc::Connection::new()?.get_workspaces()?;

    let active_workspace = workspaces
        .iter()
        .enumerate()
        .find_map(|(i, workspace)| {
            if workspace.focused {
                Some(workspaces[i].num as usize) // What?
            } else {
                None
            }
        })
        .ok_or("")?;

    Ok((active_workspace, workspaces.len()))
}
