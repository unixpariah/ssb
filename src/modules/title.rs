use hyprland::shared::HyprDataActiveOptional;

pub fn get_window_title() -> Option<String> {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    match (hyprland_running, sway_running) {
        (true, _) => Some(hyprland::data::Client::get_active().ok()??.title),
        (_, true) => {
            let workspaces = swayipc::Connection::new().ok()?.get_workspaces().ok()?;
            let active_workspace = workspaces.iter().enumerate().find_map(|(i, workspace)| {
                if workspace.focused {
                    workspaces[i].representation.clone()
                } else {
                    None
                }
            })?;

            Some(active_workspace.as_str().replace(']', "").replace("H[", ""))
        }
        _ => None,
    }
}
