use cairo::{Context, Format, ImageSurface};
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
use std::{error::Error, io::Read};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

struct Font {
    font_family: &'static str,
    font_size: f64,
    bolded: bool,
}

//                            B   G   R   A
static BACKGROUND: [u8; 4] = [33, 15, 20, 255];
static HEIGHT: i32 = 40;
static FONT: Font = Font {
    font_family: "JetBrains Mono",
    font_size: 40.0,
    bolded: true,
};

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layers: Vec<LayerSurface>,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor = CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect(
            "Failed to bind layer shell, check if the compositor supports layer shell protocol.",
        );
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");
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
                let (width, _) = self
                    .output_state()
                    .info(&output)
                    .ok_or("Failed to get output info")?
                    .logical_size
                    .ok_or("Failed to get logical size of output")?;
                let mut pool = SlotPool::new(width as usize * HEIGHT as usize * 4, &self.shm)?;
                let (buffer, canvas) =
                    pool.create_buffer(width, HEIGHT, width * 4, wl_shm::Format::Argb8888)?;

                let img = image::open("output.png").unwrap();
                let img = img.resize_exact(
                    width as u32,
                    HEIGHT as u32,
                    image::imageops::FilterType::Lanczos3,
                );
                canvas.copy_from_slice(&img.to_rgba8());

                if let Some(layer) = self.layers.get(index) {
                    layer.set_size(width as u32, HEIGHT as u32);
                    layer.set_exclusive_zone(HEIGHT);
                    layer.wl_surface().damage_buffer(0, 0, width, HEIGHT);
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
    let surface = ImageSurface::create(Format::ARgb32, 1920, HEIGHT).expect("Can't create surface");
    let context = Context::new(&surface).unwrap();
    context.set_source_rgba(
        BACKGROUND[0] as f64 / 255.0,
        BACKGROUND[1] as f64 / 255.0,
        BACKGROUND[2] as f64 / 255.0,
        BACKGROUND[3] as f64 / 255.0,
    );
    context.paint();
    context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    context.select_font_face(
        FONT.font_family,
        cairo::FontSlant::Normal,
        if FONT.bolded {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    context.set_font_size(FONT.font_size);
    context.move_to(HEIGHT as f64 / 2.0, HEIGHT as f64 - 5.0);
    context.show_text("Hello, World!").unwrap();

    let mut file = std::fs::File::create("output.png").expect("Can't create file");
    surface.write_to_png(&mut file).expect("Can't write to png");

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut bar = StatusBar::new(&globals, &qh);
    loop {
        let _ = event_queue.blocking_dispatch(&mut bar);
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
