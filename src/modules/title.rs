use hyprland::shared::HyprDataActiveOptional;

pub fn get_window_title() -> Option<Box<str>> {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    let sway_running = std::env::var("SWAYSOCK").is_ok();

    match (hyprland_running, sway_running) {
        (true, _) => Some(
            hyprland::data::Client::get_active()
                .ok()??
                .initial_title
                .into(),
        ),
        (_, true) => {
            let mut workspaces = swayipc::Connection::new().ok()?.get_workspaces().ok()?;
            let active_workspace = workspaces.iter_mut().find_map(|workspace| {
                if workspace.focused {
                    workspace.representation.take()
                } else {
                    None
                }
            })?;

            Some(active_workspace.replace(']', "").replace("H[", "").into())
        }
        _ => None,
    }
}
