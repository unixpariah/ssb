use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use std::error::Error;

pub fn workspaces(active: &'static str, inactive: &'static str) -> Result<String, Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active().unwrap().id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    Ok((0..length).fold(String::new(), |mut workspace_state, i| {
        let workspace = if i == active_workspace - 1 || i == length - 1 && active_workspace > length
        {
            active
        } else {
            inactive
        };
        workspace_state.push_str(workspace);
        workspace_state.push(' ');

        workspace_state
    }))
}
