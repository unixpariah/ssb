mod config;
mod util;

use cairo::{Context, Format, ImageSurface};
use config::{BACKGROUND, COMMAND_CONFIGS, FONT, HEIGHT, PLACEMENT, UNKOWN};
use image::RgbaImage;
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
use std::{error::Error, time::Instant};
use util::{new_command, resize_image};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

#[derive(Copy, Clone)]
pub enum CpuOpts {
    Perc,
}

#[derive(Copy, Clone)]
pub enum BacklightOpts {
    Perc,
    Value,
}

#[derive(Copy, Clone)]
pub enum RamOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

#[derive(Copy, Clone)]
pub enum Cmd {
    Custom(&'static str, &'static str),
    Workspaces(&'static str, &'static str),
    Backlight(BacklightOpts),
    Ram(RamOpts),
    Cpu(CpuOpts),
}

pub struct Font {
    family: &'static str,
    size: f64,
    bold: bool,
    color: [u8; 3],
}

struct OutputDetails {
    output_id: u32,
    layer_surface: LayerSurface,
    output: wl_output::WlOutput,
}

struct StatusData {
    output: String,
    command: Cmd,
    x: f64,
    y: f64,
    format: &'static str,
    interval: usize,
    timestamp: Instant,
}

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    outputs: Vec<OutputDetails>,
    layer_shell: LayerShell,
    compositor_state: CompositorState,
    information: Vec<StatusData>,
}

impl StatusBar {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect(
            "Failed to bind layer shell, check if the compositor supports layer shell protocol.",
        );
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");

        let information = COMMAND_CONFIGS
            .iter()
            .map(|(command, x, y, format, interval)| StatusData {
                output: get_command_output(command).unwrap_or(UNKOWN.to_string()),
                command: *command,
                x: *x,
                y: *y,
                format: *format,
                interval: *interval,
                timestamp: Instant::now(),
            })
            .collect();

        Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            outputs: Vec::new(),
            information,
        }
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.outputs.iter().try_for_each(|output| {
            let (width, _) = self
                .output_state
                .info(&output.output)
                .ok_or("Failed to get output info")?
                .logical_size
                .ok_or("Failed to get logical size of output")?;

            let mut pool = SlotPool::new(width as usize * HEIGHT as usize * 4, &self.shm)?;
            let (buffer, canvas) =
                pool.create_buffer(width, HEIGHT, width * 4, wl_shm::Format::Argb8888)?;

            let surface = ImageSurface::create(Format::ARgb32, width, HEIGHT)?;
            let context = Context::new(&surface)?;
            set_context_properties(&context);

            self.information.iter_mut().for_each(|info| {
                if info.timestamp.elapsed().as_millis() >= info.interval as u128 {
                    info.output = get_command_output(&info.command).unwrap_or(UNKOWN.to_string());
                    info.timestamp = Instant::now();
                }
                let format = info.format.replace("s%", &info.output);
                context.move_to(info.x, info.y);
                let _ = context.show_text(&format);
            });

            let mut img = Vec::new();
            surface.write_to_png(&mut img)?;

            let img = RgbaImage::from(image::load_from_memory(&img)?);
            let img = resize_image(&img, width as u32, HEIGHT as u32)?;

            canvas.copy_from_slice(&img);

            let layer = &output.layer_surface;
            layer.wl_surface().damage_buffer(0, 0, width, HEIGHT);
            layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            layer.commit();

            Ok::<(), Box<dyn Error>>(())
        })
    }
}

impl OutputHandler for StatusBar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let surface = self.compositor_state.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Bottom,
            Some("ssb"),
            Some(&output),
        );

        if let Some(info) = self.output_state.info(&output) {
            if let Some((width, _)) = info.logical_size {
                layer.set_size(width as u32, HEIGHT as u32);
                layer.set_anchor(PLACEMENT);
                layer.set_exclusive_zone(HEIGHT);
                layer.commit();

                self.outputs.push(OutputDetails {
                    output_id: info.id,
                    layer_surface: layer,
                    output,
                });
            }
        }
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
        output: wl_output::WlOutput,
    ) {
        if let Some(output_info) = self.output_state.info(&output) {
            self.outputs.retain(|info| info.output_id != output_info.id);
        }
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
        FONT.family,
        cairo::FontSlant::Normal,
        if FONT.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    context.set_font_size(FONT.size);
}

fn get_command_output(d: &Cmd) -> Result<String, Box<dyn Error>> {
    Ok(match d {
        Cmd::Custom(command, args) => new_command(command, args)?,
        Cmd::Workspaces(active, inactive) => util::get_current_workspace(active, inactive)?,
        Cmd::Ram(opt) => util::get_ram(*opt)?.split('.').next().ok_or("")?.into(),
        Cmd::Backlight(opt) => util::get_backlight(*opt)?
            .split('.')
            .next()
            .ok_or("")?
            .into(),
        Cmd::Cpu(opt) => util::get_cpu(*opt)?.split('.').next().ok_or("")?.into(),
    })
}

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init globals");
    let qh = event_queue.handle();
    let mut status_bar = StatusBar::new(&globals, &qh);
    event_queue
        .blocking_dispatch(&mut status_bar)
        .expect("Failed to dispatch events");
    loop {
        event_queue
            .blocking_dispatch(&mut status_bar)
            .expect("Failed to dispatch events");
        let _ = status_bar.draw();
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
