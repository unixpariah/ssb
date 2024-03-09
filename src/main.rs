mod config;
mod util;

use cairo::{Context, Format, ImageSurface};
use config::{BACKGROUND, DATA, FONT, HEIGHT, PLACEMENT};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::{collections::HashMap, error::Error};
use util::new_command;
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{
        wl_output::{self, WlOutput},
        wl_shm,
    },
    Connection, QueueHandle,
};

pub enum Data {
    Custom(&'static str, &'static str),
    Ram,
    Backlight,
    Cpu,
    Workspaces,
}

pub struct Font {
    font_family: &'static str,
    font_size: f64,
    bolded: bool,
    color: [u8; 3],
}

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layers: HashMap<WlOutput, LayerSurface>,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor = CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect(
            "Failed to bind layer shell, check if the compositor supports layer shell protocol.",
        );
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");
        let output_state = OutputState::new(globals, qh);

        let mut layers = HashMap::new();
        output_state.outputs().for_each(|output| {
            let surface = compositor.create_surface(qh);
            let layer = layer_shell.create_layer_surface(
                qh,
                surface,
                Layer::Bottom,
                Some("status-bar"),
                Some(&output),
            );

            layer.set_size(1, HEIGHT as u32);
            layer.set_anchor(PLACEMENT);
            layer.set_exclusive_zone(HEIGHT);
            layer.commit();

            layers.insert(output, layer);
        });

        Self {
            output_state,
            registry_state: RegistryState::new(globals),
            shm,
            layers,
        }
    }
    fn draw(&mut self) {
        let _ = self.output_state().outputs().try_for_each(|output| {
            let (width, _) = self
                .output_state()
                .info(&output)
                .ok_or("Failed to get output info")?
                .logical_size
                .ok_or("Failed to get logical size of output")?;
            let mut pool = SlotPool::new(width as usize * HEIGHT as usize * 4, &self.shm)?;
            let (buffer, canvas) =
                pool.create_buffer(width, HEIGHT, width * 4, wl_shm::Format::Argb8888)?;

            let img_surface =
                ImageSurface::create(Format::ARgb32, width, HEIGHT).expect("Can't create surface");
            let context = Context::new(&img_surface)?;
            set_context_properties(&context);

            DATA.iter().for_each(|d| {
                context.move_to(d.1, d.2);
                let format = d.3;
                let output = get_output(&d.0);
                let format = format.replace('$', &output);
                let _ = context.show_text(&format);
            });

            let mut img = Vec::new();
            img_surface
                .write_to_png(&mut img)
                .expect("Can't write to png");

            let img = image::load_from_memory(&img).expect("Can't load image");
            let img = img.resize(
                width as u32,
                HEIGHT as u32,
                image::imageops::FilterType::Lanczos3,
            );

            canvas.copy_from_slice(&img.to_rgba8().into_raw());

            if let Some(layer) = self.layers.get(&output) {
                layer.set_size(width as u32, HEIGHT as u32);
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

fn set_context_properties(context: &Context) {
    context.set_source_rgba(
        BACKGROUND[2] as f64 / 255.0,
        BACKGROUND[1] as f64 / 255.0,
        BACKGROUND[0] as f64 / 255.0,
        BACKGROUND[3] as f64 / 255.0,
    );
    let _ = context.paint();
    context.set_source_rgb(
        FONT.color[2] as f64 / 255.0,
        FONT.color[1] as f64 / 255.0,
        FONT.color[0] as f64 / 255.0,
    );
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
}

fn get_output(d: &Data) -> String {
    match d {
        Data::Custom(command, args) => String::from_utf8(new_command(command, args))
            .unwrap()
            .trim()
            .to_string(),
        Data::Ram => util::get_ram().to_string().split('.').collect::<Vec<_>>()[0].into(),
        Data::Backlight => util::get_backlight()
            .to_string()
            .split('.')
            .collect::<Vec<_>>()[0]
            .into(),
        Data::Cpu => util::get_cpu().to_string().split('.').collect::<Vec<_>>()[0].into(),
        Data::Workspaces => util::get_current_workspace()
            .unwrap_or("N/A".to_string())
            .to_string(),
    }
}

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init globals");
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
