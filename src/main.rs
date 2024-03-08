use std::error::Error;

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
    layers: Vec<LayerSurface>,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor = CompositorState::bind(globals, qh).unwrap();
        let layer_shell = LayerShell::bind(globals, qh).unwrap();
        let shm = Shm::bind(globals, qh).unwrap();
        let output_state = OutputState::new(globals, qh);

        let layers = output_state
            .outputs()
            .map(|output| {
                let surface = compositor.create_surface(qh);
                let layer = layer_shell.create_layer_surface(
                    qh,
                    surface,
                    Layer::Top,
                    Some("status-bar"),
                    Some(&output),
                );

                layer.set_size(1, 1);
                layer.set_anchor(Anchor::TOP);

                layer.commit();

                layer
            })
            .collect();

        Self {
            output_state,
            registry_state: RegistryState::new(globals),
            shm,
            layers,
        }
    }

    fn draw(&mut self) {
        let _ = self
            .output_state()
            .outputs()
            .enumerate()
            .try_for_each(|(index, output)| {
                let height = 50;
                let (width, _) = self
                    .output_state()
                    .info(&output)
                    .ok_or("Failed to get output info")?
                    .logical_size
                    .ok_or("Failed to get logical size of output")?;
                let mut pool = SlotPool::new(width as usize * height as usize * 4, &self.shm)?;
                let (buffer, canvas) =
                    pool.create_buffer(width, height, width * 4, wl_shm::Format::Argb8888)?;

                canvas.chunks_exact_mut(4).for_each(|pixel| {
                    pixel.copy_from_slice(&[0, 0, 0, 255]);
                });

                if let Some(layer) = self.layers.get(index) {
                    layer.set_size(width as u32, height as u32);
                    layer.set_exclusive_zone(height);
                    layer.wl_surface().damage_buffer(0, 0, width, height);
                    layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                    layer.commit();
                };

                Ok::<(), Box<dyn Error>>(())
            });
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
