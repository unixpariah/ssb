use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::wl_output,
    Connection, QueueHandle,
};

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        Self {
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
        }
    }
}

impl OutputHandler for StatusBar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();

    let qh = event_queue.handle();

    let mut bar = StatusBar::new(&globals, &qh);

    loop {
        event_queue.blocking_dispatch(&mut bar).unwrap();
    }
}

delegate_registry!(StatusBar);
delegate_output!(StatusBar);

impl ProvidesRegistryState for StatusBar {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
