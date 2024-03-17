mod config;
mod modules;
mod util;

use cairo::{Context, Format, ImageSurface};
use config::{COMMAND_CONFIGS, HEIGHT, TOPBAR, UNKOWN};
use modules::{
    backlight::BacklightOpts, battery::BatteryOpts, custom::get_command_output, memory::RamOpts,
};
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
use std::{collections::HashMap, error::Error, sync::mpsc};
use tokio::sync::broadcast;
use util::{
    helpers::set_context_properties,
    listeners::{Listeners, Trigger},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

#[derive(Copy, Clone, Debug)]
pub enum Cmd {
    Custom(&'static str, &'static str),
    Workspaces(&'static str, &'static str),
    Backlight(BacklightOpts),
    Ram(RamOpts),
    Cpu,
    Battery(BatteryOpts),
}

#[derive(Debug)]
struct Surface {
    output_id: u32,
    layer_surface: LayerSurface,
    width: i32,
    configured: bool,
}

pub struct StatusData {
    output: String,
    command: Cmd,
    x: f64,
    y: f64,
    format: &'static str,
    receiver: Option<broadcast::Receiver<bool>>,
    redraw: Option<mpsc::Receiver<bool>>,
}

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    surfaces: Vec<Surface>,
    layer_shell: LayerShell,
    compositor_state: CompositorState,
    information: Vec<StatusData>,
    draw: mpsc::Receiver<bool>,
    cache: HashMap<i32, (ImageSurface, Context)>,
    dispatch: bool,

    // If listeners goes out of scope hotwatch will break
    #[allow(dead_code)]
    listeners: Listeners,
}

impl StatusBar {
    fn new(
        globals: &GlobalList,
        qh: &wayland_client::QueueHandle<Self>,
        rx: mpsc::Receiver<bool>,
    ) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect(
            "Failed to bind layer shell, check if the compositor supports layer shell protocol.",
        );
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");

        let mut listeners = Listeners::new();

        let information = COMMAND_CONFIGS
            .iter()
            .map(|(command, x, y, format, event)| {
                let receiver = match event {
                    Trigger::WorkspaceChanged => listeners.new_workspace_listener(),
                    Trigger::TimePassed(interval) => listeners.new_time_passed_listener(*interval),
                    Trigger::FileChange(path) => listeners.new_file_change_listener(path),
                };

                let receiver = Some(receiver);

                StatusData {
                    output: String::new(),
                    command: *command,
                    x: *x,
                    y: *y,
                    format,
                    receiver,
                    redraw: None,
                }
            })
            .collect();

        listeners.start_time_passed_listeners();

        Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            surfaces: Vec::new(),
            information,
            listeners,
            draw: rx,
            cache: HashMap::new(),
            dispatch: true,
        }
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.surfaces.iter_mut().try_for_each(|surface| {
            if !surface.configured {
                return Ok(());
            }

            let unchanged = !self
                .information
                .iter_mut()
                .map(|info| {
                    if let Some(redraw) = &info.redraw {
                        if redraw.try_recv().is_ok() || info.output.is_empty() {
                            let output =
                                get_command_output(&info.command).unwrap_or(UNKOWN.to_string());

                            if output != info.output {
                                info.output = output;
                                return true;
                            }
                        };
                    }
                    false
                })
                .fold(false, |a, b| if b { b } else { a });

            if unchanged {
                self.dispatch = false;
                return Ok(());
            }

            let width = surface.width;
            if self.cache.get(&width).is_none() {
                let img_surface = ImageSurface::create(Format::Rgb30, width, HEIGHT)?;
                let context = Context::new(&img_surface)?;
                set_context_properties(&context);

                self.cache.insert(width, (img_surface, context));
            }

            // This will always be Some at this point
            let (img_surface, context) = self.cache.get(&width).unwrap();

            self.information.iter_mut().try_for_each(|info| {
                let format = info.format.replace("s%", &info.output);
                context.move_to(info.x, info.y);
                let _ = context.show_text(&format);
                Ok::<(), Box<dyn Error>>(())
            })?;

            let mut img = Vec::new();
            img_surface.write_to_png(&mut img)?;
            let img = image::load_from_memory(&img)?;

            let mut pool = SlotPool::new(width as usize * HEIGHT as usize * 3, &self.shm)?;
            let (buffer, canvas) =
                pool.create_buffer(width, HEIGHT, width * 3, wl_shm::Format::Bgr888)?;

            canvas.copy_from_slice(&img.to_rgb8());

            if surface.configured {
                let layer = &surface.layer_surface;
                layer.wl_surface().damage_buffer(0, 0, width, HEIGHT);
                layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                layer.wl_surface().commit();
            }

            self.dispatch = true;
            self.cache = HashMap::new();

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
                layer.set_anchor(if TOPBAR { Anchor::TOP } else { Anchor::BOTTOM });
                layer.set_exclusive_zone(HEIGHT);
                layer.set_size(width as u32, HEIGHT as u32);
                layer.commit();

                self.surfaces.push(Surface {
                    output_id: info.id,
                    layer_surface: layer,
                    width,
                    configured: false,
                });
            }
        }
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let layer = self.layer_shell.create_layer_surface(
            qh,
            self.compositor_state.create_surface(qh),
            Layer::Top,
            Some("ssb"),
            Some(&output),
        );

        // TODO: Better error handling
        let info = self.output_state.info(&output).unwrap();
        let width = info.logical_position.unwrap().0;
        let id = info.id;

        if let Some(info) = self.surfaces.iter_mut().find(|info| info.output_id == id) {
            info.width = width;
            info.configured = false;
            layer.set_size(width as u32, HEIGHT as u32);
            layer.commit();
        }
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if let Some(output_info) = self.output_state.info(&output) {
            self.surfaces
                .retain(|info| info.output_id != output_info.id);
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

async fn setup_listeners(
    listeners: Vec<(Option<broadcast::Receiver<bool>>, mpsc::Sender<bool>)>,
    sender: mpsc::Sender<bool>,
) {
    for mut listener in listeners {
        let sender = sender.clone();
        tokio::spawn(async move {
            loop {
                // This will always be Some at this point
                if let Ok(message) = listener.0.as_mut().unwrap().recv().await {
                    let _ = sender.send(message);
                    let _ = listener.1.send(true);
                };
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init globals");
    let qh = event_queue.handle();

    let (tx, rx) = mpsc::channel();

    let mut status_bar = StatusBar::new(&globals, &qh, rx);
    let mut skip = true;

    let receivers = status_bar
        .information
        .iter_mut()
        .map(|info| {
            let (tx, rx) = mpsc::channel();
            info.redraw = Some(rx);
            (info.receiver.take(), tx)
        })
        .collect();

    setup_listeners(receivers, tx).await;

    loop {
        status_bar.draw().ok();

        status_bar.surfaces.iter_mut().for_each(|surface| {
            if !surface.configured {
                surface.configured = true;
                skip = true;
            }
        });

        if status_bar.dispatch {
            event_queue
                .blocking_dispatch(&mut status_bar)
                .expect("Failed to dispatch events");
        }

        if skip {
            skip = false;
            continue;
        }

        let _ = status_bar.draw.recv();
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
