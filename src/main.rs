mod config;
mod modules;
mod surface;
mod util;

use crate::util::helpers::CSS;
use cairo::{Context, ImageSurface};
use config::{get_config, get_css, Config};
use image::{ColorType, DynamicImage};
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
        wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler},
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use std::{
    error::Error,
    sync::{mpsc, Once},
};
use surface::Surface;
use tokio::sync::broadcast;
use util::{
    helpers::TOML,
    listeners::{Listeners, Trigger},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::wl_output,
    Connection, QueueHandle,
};

// TODO: make these vecs of strings optional
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Cmd {
    Custom(String, String, Trigger, String),
    Workspaces([String; 2]),
    Backlight(String, Vec<String>),
    Memory(MemoryOpts, u64, String),
    Audio(String, Vec<String>),
    Cpu(u64, String),
    Battery(u64, String, Vec<String>),
}

pub struct ModuleData {
    output: String,
    command: Cmd,
    format: String,
    receiver: broadcast::Receiver<()>,
    cache: DynamicImage,
    position: Position,
}

#[derive(Clone)]
enum Position {
    Left,
    Center,
    Right,
}

struct StatusBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    surfaces: Vec<Surface>,
    layer_shell: LayerShell,
    compositor_state: CompositorState,
    module_info: Vec<ModuleData>,
    draw_receiver: mpsc::Receiver<()>,
    config: HotConfig,
}

struct HotConfig {
    css: String,
    css_listener: broadcast::Receiver<()>,
    config: Config,
    config_listener: broadcast::Receiver<()>,
}

static MESSAGE: &str = "If you see this, please contact lazy ass developer behind this project who did not care to update default config";
static START: Once = Once::new();

impl StatusBar {
    fn new(
        globals: &GlobalList,
        qh: &wayland_client::QueueHandle<Self>,
        rx: mpsc::Receiver<()>,
    ) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect("Failed to bind layer shell.");
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");
        let css = get_css().unwrap_or_else(|_| {
            warn!("CSS could not be parsed, using default styles");
            toml::from_str(CSS).expect(MESSAGE)
        });
        let config = get_config().unwrap_or_else(|_| {
            warn!("Config file could not be parsed, using default configuration");
            toml::from_str(TOML).expect(MESSAGE)
        });

        let mut listeners = Listeners::new();
        let positions = [
            (Position::Left, &config.modules.left),
            (Position::Center, &config.modules.center),
            (Position::Right, &config.modules.right),
        ];

        let module_info: Vec<_> = positions
            .iter()
            .flat_map(|(position, modules)| modules.iter().map(move |module| (position, module)))
            .filter_map(|(position, module)| {
                let (receiver, format) = match &module.command {
                    Cmd::Workspaces(_) => (listeners.new_workspace_listener(), "%s"),
                    Cmd::Memory(_, interval, format)
                    | Cmd::Cpu(interval, format)
                    | Cmd::Battery(interval, format, _) => {
                        if let Cmd::Battery(_, _, _) = &module.command {
                            if battery_details().is_err() {
                                warn!("Battery not found, deactivating module");
                                return None;
                            }
                        }
                        (listeners.new_time_listener(*interval), format.as_str())
                    }
                    Cmd::Backlight(format, _) => match get_backlight_path() {
                        Ok(path) => (
                            listeners.new_file_listener(&path.join("brightness")),
                            format.as_str(),
                        ),
                        Err(_) => {
                            warn!("Backlight not found, deactivating module");
                            return None;
                        }
                    },
                    Cmd::Audio(format, _) => {
                        (listeners.new_volume_change_listener(), format.as_str())
                    }
                    Cmd::Custom(_, _, trigger, format) => {
                        let trigger = match trigger {
                            Trigger::WorkspaceChanged => listeners.new_workspace_listener(),
                            Trigger::TimePassed(interval) => listeners.new_time_listener(*interval),
                            Trigger::FileChange(path) => listeners.new_file_listener(path),
                            Trigger::VolumeChanged => listeners.new_volume_change_listener(),
                        };
                        (trigger, format.as_str())
                    }
                };

                Some(ModuleData {
                    output: String::new(),
                    command: module.command.to_owned(),
                    format: format.to_string(),
                    receiver,
                    cache: DynamicImage::new(0, 0, ColorType::L8),
                    position: position.clone(),
                })
            })
            .collect();

        let config_dir = dirs::config_dir().expect("Failed to get config directory");
        let css_path = config_dir.join(format!("{}/style.css", env!("CARGO_PKG_NAME")));
        let config_path = config_dir.join("config.toml");
        let config = HotConfig {
            css,
            css_listener: listeners.new_file_listener(&css_path),
            config,
            config_listener: listeners.new_file_listener(&config_path),
        };

        listeners.start_all();

        Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            surfaces: Vec::new(),
            module_info,
            draw_receiver: rx,
            config,
        }
    }

    fn reload_config(&mut self) -> bool {
        let mut config_changed = false;
        if self.config.css_listener.try_recv().is_ok() {
            self.config.css = get_css().unwrap_or_else(|_| {
                warn!("CSS could not be parsed, using default styles");
                toml::from_str(CSS).expect(MESSAGE)
            });
            config_changed = true;
        }
        if self.config.config_listener.try_recv().is_ok() {
            self.config.config = get_config().unwrap_or_else(|_| {
                warn!("Config file could not be parsed, using default configuration");
                toml::from_str(TOML).expect(MESSAGE)
            });
            std::mem::take(&mut self.config.config.modules); // Drop it as its not gonna be used anymore
            let anchor = match self.config.config.topbar {
                true => Anchor::TOP,
                false => Anchor::BOTTOM,
            };
            self.surfaces.iter_mut().for_each(|surface| {
                surface.layer_surface.set_anchor(anchor);
                surface
                    .layer_surface
                    .set_size(surface.width as u32, self.config.config.height as u32);
                surface
                    .layer_surface
                    .set_exclusive_zone(self.config.config.height);
            });
            config_changed = true;
        };

        !self
            .module_info
            .par_iter_mut()
            .map(|info| {
                if info.receiver.try_recv().is_ok() || info.output.is_empty() || config_changed {
                    let output = get_command_output(&info.command)
                        .unwrap_or(self.config.config.unkown.to_string());

                    if output != info.output || config_changed {
                        let format = info.format.replace("%s", &output);
                        let format = match &info.command {
                            Cmd::Battery(_, _, icons)
                            | Cmd::Backlight(_, icons)
                            | Cmd::Audio(_, icons)
                                if !icons.is_empty() =>
                            {
                                if let Ok(output) = output.parse::<usize>() {
                                    let range_size = 100 / icons.len();
                                    let icon =
                                        &icons[std::cmp::min(output / range_size, icons.len() - 1)];
                                    format.replace("%c", icon)
                                } else {
                                    format.replace("%c", "")
                                }
                            }
                            _ => format,
                        };

                        let name = match &info.command {
                            Cmd::Workspaces(_) => "workspaces",
                            Cmd::Memory(_, _, _) => "memory",
                            Cmd::Cpu(_, _) => "cpu",
                            Cmd::Battery(_, _, _) => "battery",
                            Cmd::Backlight(_, _) => "backlight",
                            Cmd::Audio(_, _) => "audio",
                            Cmd::Custom(_, name, _, _) => name,
                        };

                        let css = &self.config.css;
                        let css_section = match css.find(name) {
                            Some(index) => {
                                let end_index = css[index..].find('}').map(|i| i + index).unwrap();
                                css[index..end_index].to_string()
                            },
                            None => {
                                warn!("Style declaration for module {name} not found, using default style");
                                let index = CSS.find(name).expect(MESSAGE);
                                let end_index = CSS[index..].find('}').map(|i| i + index).expect(MESSAGE);
                                CSS[index..end_index].to_string()
                            }
                        };

                        let css_section = format!("{} content: \"{}\"; }}", css_section, format);
                        let img = css_image::parse(css_section.clone()).unwrap_or_else(|_| {
                            warn!("Failed to parse {name} module css, using default style");
                            let index = CSS.find(name).expect(MESSAGE);
                            let end_index =
                                CSS[index..].find('}').map(|i| i + index).expect(MESSAGE);
                            let mut css = CSS[index..end_index].to_string();
                            css.push_str(&format!(" content: \"{}\"; }}", format));
                            css_image::parse(css).expect(MESSAGE)
                        });

                        if let Ok(img) = image::load_from_memory(img.get(name).unwrap()) {
                            info.cache = img;
                        }

                        info.output = output;
                        return true;
                    }
                };

                false
            }).reduce_with(|a, b| a || b).unwrap_or(false)
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
            Some(env!("CARGO_PKG_NAME")),
            Some(&output),
        );

        if let Some(info) = self.output_state.info(&output) {
            let config = &self.config.config;
            let height = config.height;
            if let Some((width, _)) = info.logical_size {
                layer.set_anchor(match config.topbar {
                    true => Anchor::TOP,
                    false => Anchor::BOTTOM,
                });

                layer.set_exclusive_zone(height);
                layer.set_size(width as u32, height as u32);
                layer.commit();

                let img_surface =
                    ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
                let context = Context::new(&img_surface).unwrap();

                let background = config.background;

                context.set_source_rgba(
                    background[0] as f64 / 255.0,
                    background[1] as f64 / 255.0,
                    background[2] as f64 / 255.0,
                    background[3] as f64 / 255.0,
                );
                _ = context.paint();

                let mut background = Vec::new();
                _ = img_surface.write_to_png(&mut background);

                if let Ok(background) = image::load_from_memory(&background) {
                    if let Some(ref name) = info.name {
                        info!("Bar configured for output: {:?}", name);
                    }

                    self.surfaces.push(Surface {
                        output_info: info,
                        layer_surface: layer,
                        width: 0,
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
                info.output_info.id != output_info.id
            });
        }
    }
}

impl LayerShellHandler for StatusBar {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.surfaces
            .iter_mut()
            .find(|surface| &surface.layer_surface == layer)
            .unwrap()
            .change_size();
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

fn setup_listeners(
    listeners: Vec<(broadcast::Receiver<()>, broadcast::Sender<()>)>,
    sender: mpsc::Sender<()>,
) {
    listeners.into_iter().for_each(|mut listener| {
        let sender = sender.clone();
        tokio::spawn(async move {
            loop {
                if listener.0.recv().await.is_ok() {
                    _ = sender.send(());
                    _ = listener.1.send(());
                };
            }
        });
    })
}

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();
    logger();

    let conn = Connection::connect_to_env().expect("Failed to connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init globals");
    let qh = event_queue.handle();
    let (tx, rx) = mpsc::channel();
    let mut status_bar = StatusBar::new(&globals, &qh, rx);

    let mut receivers = status_bar
        .module_info
        .iter_mut()
        .map(|info| {
            let (tx, rx) = broadcast::channel(1);
            (std::mem::replace(&mut info.receiver, rx), tx)
        })
        .collect::<Vec<_>>();

    {
        let (tx, rx) = broadcast::channel(1);
        receivers.push((
            std::mem::replace(&mut status_bar.config.css_listener, rx),
            tx,
        ));
        let (tx, rx) = broadcast::channel(1);
        receivers.push((
            std::mem::replace(&mut status_bar.config.config_listener, rx),
            tx,
        ));
    }

    setup_listeners(receivers, tx);
    loop {
        status_bar.reload_config();
        let drawn = status_bar
            .surfaces
            .par_iter_mut()
            .map(|surface| {
                if surface.is_configured() {
                    surface
                        .draw(&status_bar.config, &status_bar.module_info, &qh, &globals)
                        .unwrap();
                    return true;
                }
                false
            })
            .reduce_with(|a, b| a || b)
            .unwrap_or(false);

        event_queue
            .blocking_dispatch(&mut status_bar)
            .expect("Failed to dispatch events");

        if drawn {
            START.call_once(|| {
                info!("Startup time: {:?}", start_time.elapsed());
            });

            status_bar
                .draw_receiver
                .recv()
                .expect("Failed to receive draw message");
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
