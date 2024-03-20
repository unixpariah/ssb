use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use std::error::Error;

pub fn workspaces(workspace: &String) -> Result<String, Box<dyn Error>> {
    let active_workspace = hyprland::data::Workspace::get_active().unwrap().id as usize;
    let length = hyprland::data::Workspaces::get()?.iter().count();

    let workspace = workspace.split_whitespace().collect::<Vec<&str>>();
    let active = workspace[0];
    let inactive = workspace[1];

    Ok((0..length).fold(String::new(), |mut workspace_state, i| {
        let workspace = if i == active_workspace - 1 || i == length - 1 && active_workspace > length
        {
            active
        } else {
            inactive
        };
        workspace_state.push_str(workspace);
        workspace_state.push_str("  ");

        workspace_state
    }))
}
