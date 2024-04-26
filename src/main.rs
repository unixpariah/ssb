mod config;
mod modules;
mod surface;
mod util;

use crate::util::helpers::CSS_STRING;
use cairo::{Context, ImageSurface};
use config::{get_config, get_css, Config};
use css_image::style::Style;
use lazy_static::lazy_static;
use log::{info, warn, LevelFilter};
use modules::{custom::Cmd, ModuleData};
use rayon::prelude::*;
use simplelog::{ColorChoice, TermLogger, TerminalMode, ThreadLogMode};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{calloop, calloop_wayland_source::WaylandSource},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{pointer::PointerHandler, SeatHandler, SeatState},
    shell::{
        wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::{
    collections::HashMap,
    error::Error,
    sync::{mpsc, Arc, Once},
};
use surface::Surface;
use tokio::sync::broadcast;
use util::{helpers::TOML_STRING, listeners::Listeners};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm},
    Connection, QueueHandle,
};

lazy_static! {
    pub static ref CSS: Arc<[Style]> = css_image::parse(CSS_STRING).expect(MESSAGE).into();
    pub static ref TOML: Arc<Config> = Arc::new(toml::from_str(TOML_STRING).expect(MESSAGE));
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
    first_run: bool,
    seat_state: SeatState,
}

pub struct HotConfig {
    pub css: Vec<Style>,
    pub css_listener: broadcast::Receiver<()>,
    pub config: Arc<Config>,
    pub config_listener: broadcast::Receiver<()>,
}

static MESSAGE: &str = "If you see this, please contact lazy ass developer behind this project who did not care to update default config";
static START: Once = Once::new();

impl StatusBar {
    fn new(
        globals: &GlobalList,
        qh: &wayland_client::QueueHandle<Self>,
        draw_receiver: mpsc::Receiver<()>,
    ) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect("Failed to bind layer shell.");
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");
        let css_str = get_css().unwrap_or("".into());
        let seat_state = SeatState::new(globals, qh);

        let css = css_image::parse(&css_str).unwrap_or_else(|_| {
            warn!("CSS could not be parsed, using default styles");
            CSS.to_owned().to_vec()
        });

        let config = get_config().unwrap_or_else(|_| {
            warn!("Config file could not be parsed, using default configuration");
            TOML.clone()
        });

        let mut listeners = Listeners::new();
        let positions = [
            (Arc::new(Position::Left), &config.modules.left),
            (Arc::new(Position::Center), &config.modules.center),
            (Arc::new(Position::Right), &config.modules.right),
        ];

        let module_info = positions
            .iter()
            .flat_map(|(position, modules)| modules.iter().map(move |module| (position, module)))
            .filter_map(|(position, module)| {
                ModuleData::new(&mut listeners, module, position.clone())
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
            draw_receiver,
            config,
            first_run: true,
            seat_state,
        }
    }

    #[inline]
    fn reload_config(&mut self) {
        let mut config_changed = false;
        if self.config.css_listener.try_recv().is_ok() {
            let css_str = get_css().unwrap_or("".into());

            self.config.css = css_image::parse(&css_str).unwrap_or_else(|_| {
                warn!("CSS could not be parsed, using default styles");
                CSS.to_owned().to_vec()
            });

            config_changed = true;
        }
        if self.config.config_listener.try_recv().is_ok() {
            self.config.config = get_config().unwrap_or_else(|_| {
                warn!("Config file could not be parsed, using default configuration");
                TOML.clone()
            });
            let anchor = match self.config.config.topbar {
                true => Anchor::TOP,
                false => Anchor::BOTTOM,
            };
            self.surfaces.iter_mut().for_each(|surface| {
                surface.create_background(&self.config.config);
                surface.layer_surface.set_anchor(anchor);
                surface
                    .layer_surface
                    .set_layer(match &*self.config.config.layer {
                        "overlay" => Layer::Overlay,
                        "background" => Layer::Background,
                        _ => Layer::Overlay,
                    });
                surface
                    .layer_surface
                    .set_size(surface.width as u32, self.config.config.height as u32);
                surface
                    .layer_surface
                    .set_exclusive_zone(self.config.config.height);
            });
            config_changed = true;
        };

        let config = &self.config;
        self.module_info.par_iter_mut().for_each(|info| {
            if info.receiver.try_recv().is_ok() || info.output.is_empty() || config_changed {
                info.render(config_changed, config);
            };
        });
    }
}

fn get_style(
    css: &[Style],
    name: &str,
    format: &str,
) -> Result<HashMap<Box<str>, Vec<u8>>, Box<dyn Error + Sync + Send>> {
    let styles = if !css.iter().any(|style| style.selector == name.into())
        && css.iter().any(|a| a.selector == "*".into())
    {
        css.iter()
            .find(|style| style.selector == "*".into())
            .cloned()
    } else {
        css.iter()
            .find(|style| style.selector == name.into())
            .cloned()
    };

    match styles {
        Some(mut style) => {
            style.content = Some(format.into());
            css_image::render(style).map_err(|_| {
                warn!("Failed to parse {name} module css, using default style");
                "".into()
            })
        }
        None => {
            warn!("Style declaration for module {name} not found, using default style");
            Err("".into())
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
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let config = &self.config.config;
        let surface = self.compositor_state.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            match &*config.layer {
                "overlay" => Layer::Overlay,
                "background" => Layer::Background,
                _ => Layer::Overlay,
            },
            Some(env!("CARGO_PKG_NAME")),
            Some(&output),
        );

        if let Some(info) = self.output_state.info(&output) {
            let height = config.height;
            if let Some((width, _)) = info.logical_size {
                layer.set_anchor(match config.topbar {
                    true => Anchor::TOP,
                    false => Anchor::BOTTOM,
                });

                layer.set_exclusive_zone(height);
                layer.set_size(width as u32, height as u32);
                layer.commit();

                if let Some(ref name) = info.name {
                    info!("Bar configured for output: {:?}", name);
                }

                let height = config.height;

                let img_surface =
                    ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
                let context = Context::new(&img_surface).unwrap();
                let background = config.background;
                context.set_source_rgba(
                    background[0] / 255.0,
                    background[1] / 255.0,
                    background[2] / 255.0,
                    background[3],
                );
                _ = context.paint();
                let mut background = Vec::new();
                _ = img_surface.write_to_png(&mut background);
                let background = image::load_from_memory(&background).unwrap();

                self.surfaces.push(Surface {
                    output_info: info,
                    layer_surface: layer,
                    width: 0,
                    background,
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

impl PointerHandler for StatusBar {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        _events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
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

impl SeatHandler for StatusBar {
    fn seat_state(&mut self) -> &mut smithay_client_toolkit::seat::SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn remove_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
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
    ping: calloop::ping::Ping,
) {
    listeners.into_iter().for_each(|mut listener| {
        let ping = ping.clone();
        let sender = sender.clone();
        tokio::spawn(async move {
            loop {
                if listener.0.recv().await.is_ok() {
                    _ = sender.send(());
                    _ = listener.1.send(());
                    ping.ping();
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
    let (globals, event_queue) = registry_queue_init(&conn).expect("Failed to init globals");
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

    let mut event_loop =
        calloop::EventLoop::<StatusBar>::try_new().expect("Failed to create event loop");
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .expect("Failed to insert wayland source");

    let (ping, ping_source) =
        calloop::ping::make_ping().expect("Unable to create a calloop::ping::Ping");
    event_loop
        .handle()
        .insert_source(ping_source, |_, _, _| {})
        .expect("Failed to insert source");

    setup_listeners(receivers, tx, ping);
    loop {
        if status_bar.draw_receiver.try_recv().is_ok() || status_bar.first_run {
            status_bar.reload_config();
            let drawn = status_bar
                .surfaces
                .par_iter_mut()
                .map(|surface| {
                    if surface.is_configured() {
                        let width = surface.width;
                        let height = status_bar.config.config.height;
                        if let Ok(mut pool) =
                            SlotPool::new((width * height * 4) as usize, &status_bar.shm)
                        {
                            if let Ok((buffer, canvas)) = pool.create_buffer(
                                width,
                                height,
                                width * 4,
                                wl_shm::Format::Abgr8888,
                            ) {
                                return surface
                                    .draw(
                                        &status_bar.config.config,
                                        &status_bar.module_info,
                                        &buffer,
                                        canvas,
                                    )
                                    .is_ok();
                            }
                        }
                    }
                    false
                })
                .reduce_with(|a, b| a || b)
                .unwrap_or(false);

            if drawn {
                status_bar.first_run = false;
                START.call_once(|| {
                    info!("Startup time: {:#?}", start_time.elapsed());
                });
            }
        }

        event_loop
            .dispatch(None, &mut status_bar)
            .expect("Failed to dispatch events");
    }
}

delegate_registry!(StatusBar);
delegate_output!(StatusBar);
delegate_layer!(StatusBar);
delegate_compositor!(StatusBar);
delegate_shm!(StatusBar);
delegate_pointer!(StatusBar);
delegate_seat!(StatusBar);

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
