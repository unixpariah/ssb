mod config;
mod modules;
mod util;

use cairo::{Context, ImageSurface};
use config::get_config;
use image::{imageops, ColorType, DynamicImage, RgbImage};
use log::{info, warn, LevelFilter};
use modules::{
    backlight::get_backlight_path, battery::battery_details, custom::get_command_output,
    memory::MemoryOpts,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, TermLogger, TerminalMode, ThreadLogMode};
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
    helpers::{get_context, set_info_context, TOML},
    listeners::{Listeners, Trigger},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

// TODO: make these vecs of strings optional
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Cmd {
    Custom(String, Trigger, String),
    Workspaces([String; 2]),
    Backlight(String, Vec<String>),
    Memory(MemoryOpts, u64, String),
    Audio(String, Vec<String>),
    Cpu(u64, String),
    Battery(u64, String, Vec<String>),
}

#[derive(Debug)]
struct Surface {
    output_id: u32,
    layer_surface: LayerSurface,
    width: i32,
    configured: bool,
    background: DynamicImage,
}

struct ImgCache {
    img: DynamicImage,
    width: i32,
    height: i32,
    unchanged: bool,
}

impl ImgCache {
    fn new(img: DynamicImage, width: i32, height: i32, unchanged: bool) -> Self {
        Self {
            img,
            width,
            height,
            unchanged,
        }
    }
}

pub struct ModuleData {
    output: String,
    command: Cmd,
    x: f64,
    y: f64,
    format: String,
    receiver: Option<broadcast::Receiver<bool>>,
    redraw: Option<mpsc::Receiver<bool>>,
    cache: ImgCache,
}

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    surfaces: Vec<Surface>,
    layer_shell: LayerShell,
    compositor_state: CompositorState,
    information: Vec<ModuleData>,
    draw: mpsc::Receiver<bool>,
    cache: HashMap<i32, RgbImage>,
    dispatch: bool,
    config: config::Config,
}

impl StatusBar {
    fn new(
        globals: &GlobalList,
        qh: &wayland_client::QueueHandle<Self>,
        rx: mpsc::Receiver<bool>,
    ) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect("Failed to bind layer shell.");
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");

        let config = get_config().unwrap_or_else(|_| {
            toml::from_str(TOML).expect("If you see this, it means that lazy ass developer did not care to update default config")
        });
        let mut listeners = Listeners::new();

        let information = config
            .modules
            .iter()
            .filter_map(|module| {
                let receiver = match &module.command {
                    Cmd::Workspaces(_) => listeners.new_workspace_listener(),
                    Cmd::Memory(_, interval, _) => listeners.new_time_passed_listener(*interval),
                    Cmd::Cpu(interval, _) => listeners.new_time_passed_listener(*interval),
                    Cmd::Battery(interval, _, _) => {
                        if battery_details().is_err() {
                            warn!("Battery not found, deactivating module");
                            return None;
                        }
                        listeners.new_time_passed_listener(*interval)
                    }
                    Cmd::Backlight(_, _) => {
                        if let Ok(path) = get_backlight_path() {
                            listeners.new_file_change_listener(&path)
                        } else {
                            warn!("Backlight not found, deactivating module");
                            return None;
                        }
                    }
                    Cmd::Audio(_, _) => listeners.new_volume_change_listener(),
                    Cmd::Custom(_, trigger, _) => match trigger {
                        Trigger::WorkspaceChanged => listeners.new_workspace_listener(),
                        Trigger::TimePassed(interval) => {
                            listeners.new_time_passed_listener(*interval)
                        }
                        Trigger::FileChange(path) => listeners.new_file_change_listener(path),
                        Trigger::VolumeChanged => listeners.new_volume_change_listener(),
                    },
                };

                let format = match &module.command {
                    Cmd::Workspaces(_) => "%s",
                    Cmd::Memory(_, _, format) => format,
                    Cmd::Cpu(_, format) => format,
                    Cmd::Battery(_, format, _) => format,
                    Cmd::Backlight(format, _) => format,
                    Cmd::Custom(_, _, format) => format,
                    Cmd::Audio(format, _) => format,
                };

                let receiver = Some(receiver);

                Some(ModuleData {
                    output: String::new(),
                    // TODO: Get rid of this clone
                    command: module.command.clone(),
                    x: module.x,
                    y: module.y,
                    format: format.to_string(),
                    receiver,
                    redraw: None,
                    cache: ImgCache::new(DynamicImage::new(0, 0, ColorType::L8), 0, 0, false),
                })
            })
            .collect();

        listeners.start_listeners();

        Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            surfaces: Vec::new(),
            information,
            draw: rx,
            cache: HashMap::new(),
            dispatch: true,
            config,
        }
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        if self.surfaces.iter().any(|surface| !surface.configured) || self.surfaces.is_empty() {
            return Ok(());
        }

        let font = &self.config.font;

        let surface = ImageSurface::create(cairo::Format::Rgb30, 0, 0).unwrap();
        let context = cairo::Context::new(&surface).unwrap();

        context.select_font_face(
            &font.family,
            cairo::FontSlant::Normal,
            if font.bold {
                cairo::FontWeight::Bold
            } else {
                cairo::FontWeight::Normal
            },
        );
        context.set_font_size(font.size);
        let unchanged = !self
            .information
            .par_iter_mut()
            .map(|info| {
                if let Some(redraw) = &info.redraw {
                    if redraw.try_recv().is_ok() || info.output.is_empty() {
                        let output = get_command_output(&info.command)
                            .unwrap_or(self.config.unkown.to_string());

                        if output != info.output {
                            let format = info.format.replace("%s", &output);
                            let format = match &info.command {
                                Cmd::Battery(_, _, icons)
                                | Cmd::Backlight(_, icons)
                                | Cmd::Audio(_, icons)
                                    if !icons.is_empty() =>
                                {
                                    if let Ok(output) = output.parse::<usize>() {
                                        let range_size = 100 / icons.len();
                                        let icon = &icons
                                            [std::cmp::min(output / range_size, icons.len() - 1)];
                                        format.replace("%c", icon)
                                    } else {
                                        format.replace("%c", "")
                                    }
                                }
                                _ => format,
                            };
                            let context = get_context(font);
                            let extents = match context.text_extents(&format) {
                                Ok(extents) => extents,
                                Err(_) => {
                                    return false;
                                }
                            };
                            let width = if extents.width() as i32 > info.cache.width {
                                extents.width() as i32
                            } else {
                                info.cache.width
                            };

                            let height = if extents.height() as i32 > info.cache.height {
                                extents.height() as i32
                            } else {
                                info.cache.height
                            };

                            let surface =
                                match ImageSurface::create(cairo::Format::Rgb30, width, height) {
                                    Ok(surface) => surface,
                                    Err(_) => {
                                        return false;
                                    }
                                };
                            let context = match cairo::Context::new(&surface) {
                                Ok(context) => context,
                                Err(_) => {
                                    return false;
                                }
                            };
                            set_info_context(&context, extents, &self.config);

                            _ = context.show_text(&format);

                            let mut img = Vec::new();
                            _ = surface.write_to_png(&mut img);

                            if let Ok(img) = image::load_from_memory(&img) {
                                info.cache = ImgCache::new(img, width, height, false);
                            }

                            info.output = output;
                            return true;
                        }
                    };
                }

                info.cache.unchanged = true;
                false
            })
            .reduce_with(|a, b| if b { b } else { a })
            .unwrap_or(false);

        if unchanged {
            self.dispatch = false;
            return Ok(());
        }

        self.cache = HashMap::new();
        self.surfaces.iter_mut().try_for_each(|surface| {
            let width = surface.width;
            let height = self.config.height;

            if self.cache.get(&width).is_none() {
                let background = &mut surface.background;
                self.information.iter().for_each(|info| {
                    if info.cache.unchanged {
                        return;
                    }

                    let img = &info.cache;
                    imageops::overlay(
                        background,
                        &img.img,
                        info.x as i64,
                        info.y as i64 - img.height as i64 / 2,
                    );
                });

                self.cache.insert(width, background.to_rgb8());
            }

            // This will always be Some at this point
            let img = self.cache.get(&width).unwrap();

            let mut pool = SlotPool::new(width as usize * height as usize * 3, &self.shm)?;
            let (buffer, canvas) =
                pool.create_buffer(width, height, width * 3, wl_shm::Format::Bgr888)?;

            canvas.copy_from_slice(img);

            if surface.configured {
                let layer = &surface.layer_surface;
                layer.wl_surface().damage_buffer(0, 0, width, height);
                layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                layer.wl_surface().commit();
            }

            self.dispatch = true;
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
            let height = self.config.height;
            if let Some((width, _)) = info.logical_size {
                layer.set_anchor(match self.config.topbar {
                    true => Anchor::TOP,
                    false => Anchor::BOTTOM,
                });

                layer.set_exclusive_zone(height);
                layer.set_size(width as u32, height as u32);
                layer.commit();

                let img_surface =
                    ImageSurface::create(cairo::Format::Rgb30, width, height).unwrap();
                let context = Context::new(&img_surface).unwrap();

                let background = self.config.background;
                let font = &self.config.font;

                context.set_source_rgb(
                    background[0] as f64 / 255.0,
                    background[1] as f64 / 255.0,
                    background[2] as f64 / 255.0,
                );
                _ = context.paint();
                context.set_source_rgb(
                    font.color[0] as f64 / 255.0,
                    font.color[1] as f64 / 255.0,
                    font.color[2] as f64 / 255.0,
                );
                context.select_font_face(
                    &font.family,
                    cairo::FontSlant::Normal,
                    if font.bold {
                        cairo::FontWeight::Bold
                    } else {
                        cairo::FontWeight::Normal
                    },
                );
                context.set_font_size(font.size);

                let mut background = Vec::new();
                _ = img_surface.write_to_png(&mut background);

                if let Ok(background) = image::load_from_memory(&background) {
                    if let Some(name) = info.name {
                        info!("Bar configured for output: {:?}", name);
                    }

                    self.surfaces.push(Surface {
                        output_id: info.id,
                        layer_surface: layer,
                        width,
                        configured: false,
                        background,
                    });
                }
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
            self.surfaces.retain(|info| {
                info!(
                    "Removing bar from output: {:?}",
                    output_info.to_owned().name.unwrap()
                );
                info.output_id != output_info.id
            });
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
    listeners.into_iter().for_each(|mut listener| {
        let sender = sender.clone();
        tokio::spawn(async move {
            loop {
                // Listener is Option just to make it easier to take the value so unwraping is safe
                if let Ok(message) = listener.0.as_mut().unwrap().recv().await {
                    _ = sender.send(message);
                    _ = listener.1.send(true);
                };
            }
        });
    })
}

#[tokio::main]
async fn main() {
    logger();

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
        .collect::<Vec<_>>();

    if !receivers.is_empty() {
        setup_listeners(receivers, tx).await;
    }

    loop {
        status_bar.draw().expect("Failed to draw");
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

        status_bar
            .draw
            .recv()
            .expect("Failed to receive draw message");
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

fn logger() {
    let config = simplelog::ConfigBuilder::new()
        .set_thread_level(LevelFilter::Error)
        .set_thread_mode(ThreadLogMode::Both)
        .build();

    TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Stderr,
        ColorChoice::AlwaysAnsi,
    )
    .expect("Failed to initialize logger");
}
