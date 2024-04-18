use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use hyprland::shared::HyprDataActiveOptional;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WorkspacesIcons {
    pub active: String,
    pub inactive: String,
}

pub fn get_window_title() -> Option<String> {
    Some(hyprland::data::Client::get_active().ok()??.title)
}

pub fn workspaces(workspace: &WorkspacesIcons) -> String {
    let (active_workspace, length) = match hyprland() {
        Ok((active, len)) => (Some(active), Some(len)),
        Err(_) => (None, None),
    };

    let active_workspace = active_workspace.unwrap();
    let length = length.unwrap();

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

fn hyprland() -> Result<(usize, usize), Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active()?.id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    Ok((active_workspace, length))
}
