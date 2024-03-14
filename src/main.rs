mod config;
mod util;

use cairo::{Context, Format, ImageSurface};
use config::{COMMAND_CONFIGS, HEIGHT, PLACEMENT, UNKOWN};
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
use std::{error::Error, sync::mpsc::Receiver, thread, time::Duration};
use util::{
    create_file_change_listener, create_time_passed_listener, create_workspace_listener,
    get_command_output, set_context_properties, BacklightOpts, RamOpts, Trigger,
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

#[derive(Copy, Clone)]
pub enum Cmd {
    Custom(&'static str, &'static str),
    Workspaces(&'static str, &'static str),
    Backlight(BacklightOpts),
    Ram(RamOpts),
    Cpu,
}

struct OutputDetails {
    output_id: u32,
    layer_surface: LayerSurface,
    output: wl_output::WlOutput,
}

pub struct StatusData {
    output: String,
    command: Cmd,
    x: f64,
    y: f64,
    format: &'static str,
    receiver: Receiver<bool>,
    redraw: bool,
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
            .map(|(command, x, y, format, event)| {
                let receiver = match event {
                    Trigger::WorkspaceChanged => create_workspace_listener(),
                    Trigger::TimePassed(interval) => create_time_passed_listener(*interval),
                    Trigger::FileChange(path) => create_file_change_listener(path),
                };

                StatusData {
                    output: get_command_output(command).unwrap_or(UNKOWN.to_string()),
                    command: *command,
                    x: *x,
                    y: *y,
                    format,
                    receiver,
                    redraw: true,
                }
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
        self.outputs.iter_mut().try_for_each(|output| {
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

            self.information.iter_mut().try_for_each(|info| {
                if info.redraw {
                    info.output = get_command_output(&info.command)?;
                    info.redraw = false;
                }

                let format = info.format.replace("s%", &info.output);
                context.move_to(info.x, info.y);
                let _ = context.show_text(&format);
                Ok::<(), Box<dyn Error>>(())
            })?;

            let mut img = Vec::new();
            surface.write_to_png(&mut img)?;
            let img = RgbaImage::from(image::load_from_memory(&img)?);

            canvas.copy_from_slice(&img);

            let layer = &output.layer_surface;
            layer.wl_surface().damage_buffer(0, 0, width, HEIGHT);
            layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            layer.wl_surface().commit();
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
            Layer::Top,
            Some("ssb"),
            Some(&output),
        );

        if let Some(info) = self.output_state.info(&output) {
            if let Some((width, _)) = info.logical_size {
                layer.set_anchor(PLACEMENT);
                layer.set_exclusive_zone(HEIGHT);
                layer.set_size(width as u32, HEIGHT as u32);
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

        status_bar.draw().expect("Failed to draw status bar");

        loop {
            let break_loop = status_bar.information.iter_mut().any(|info| {
                if let Ok(rx) = info.receiver.try_recv() {
                    info.redraw = rx;
                    return rx;
                }

                false
            });

            if break_loop {
                break;
            }

            thread::sleep(Duration::from_millis(1));
        }
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
