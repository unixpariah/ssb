use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor = CompositorState::bind(globals, qh).unwrap();
        let layer_shell = LayerShell::bind(globals, qh).unwrap();
        let surface = compositor.create_surface(qh);
        let shm = Shm::bind(globals, qh).unwrap();

        let layer =
            layer_shell.create_layer_surface(qh, surface, Layer::Top, Some("status-bar"), None);

        layer.set_size(200, 200);
        layer.set_anchor(Anchor::TOP);
        layer.set_exclusive_zone(1);

        layer.commit();

        Self {
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            layer,
        }
    }

    fn draw(&mut self) {
        let mut pool = SlotPool::new(200 * 200 * 4, &self.shm).unwrap();
        let (buffer, canvas) = pool
            .create_buffer(200, 200, 200 * 4, wl_shm::Format::Argb8888)
            .unwrap();

        canvas.chunks_exact_mut(4).for_each(|pixel| {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        });

        self.layer.set_size(200, 200);
        self.layer.wl_surface().damage_buffer(0, 0, 200, 200);
        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer.commit();
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

impl LayerShellHandler for StatusBar {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
    }

    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
    }
}

impl CompositorHandler for StatusBar {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
    }
}

impl ShmHandler for StatusBar {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut bar = StatusBar::new(&globals, &qh);
    loop {
        event_queue.blocking_dispatch(&mut bar).unwrap();
        bar.draw();
    }
}

delegate_registry!(StatusBar);
delegate_output!(StatusBar);
delegate_layer!(StatusBar);
delegate_compositor!(StatusBar);
delegate_shm!(StatusBar);

impl ProvidesRegistryState for StatusBar {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
