extern crate libpulse_binding as pulse;

use hyprland::event_listener::EventListener;
use inotify::{Inotify, WatchMask};
use log::warn;
use pulse::{
    context::{subscribe::InterestMaskSet, Context, FlagSet as ContextFlagSet},
    mainloop::threaded::Mainloop,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};
use swayipc::EventType;
use tokio::sync::broadcast;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(PathBuf),
    VolumeChanged,
}

pub enum WorkspaceListener {
    Hyprland(Box<EventListener>),
    Sway(JoinHandle<()>),
}

pub struct WorkspaceListenerData {
    tx: broadcast::Sender<()>,
    listener: WorkspaceListener,
}

impl WorkspaceListenerData {
    pub fn new() -> Result<Self, Box<dyn crate::Error>> {
        let tx = broadcast::Sender::new(1);

        let hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE");
        let sway = std::env::var("SWAYSOCK");

        let listener = match (hyprland.is_ok(), sway.is_ok()) {
            (true, _) => {
                let mut listener = EventListener::new();
                {
                    let tx = tx.clone();
                    listener.add_workspace_destroy_handler(move |_| {
                        _ = tx.send(());
                    });
                }

                {
                    let tx = tx.clone();
                    listener.add_workspace_change_handler(move |_| {
                        _ = tx.send(());
                    });
                }

                {
                    let tx = tx.clone();
                    listener.add_active_monitor_change_handler(move |_| {
                        _ = tx.send(());
                    });
                }

                WorkspaceListener::Hyprland(Box::new(listener))
            }
            (_, true) => {
                let listener = swayipc::Connection::new()?
                    .subscribe([EventType::Workspace, EventType::Output])?;

                let tx = tx.clone();
                let handle = thread::spawn(move || {
                    listener.for_each(|_| {
                        _ = tx.send(());
                    });
                });

                WorkspaceListener::Sway(handle)
            }
            _ => {
                warn!("Unsupported compositor, disabling workspaces module");
                return Err("No supported compositor found".into());
            }
        };

        Ok(Self { tx, listener })
    }
}

pub struct TimeListenerData {
    tx: broadcast::Sender<()>,
    interval: u64,
    original_interval: u64,
}

pub struct FileListenerData {
    store: HashMap<Box<str>, broadcast::Sender<()>>,
    inotify: Inotify,
}

impl FileListenerData {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
            inotify: Inotify::init().expect("Failed to setup inotify"),
        }
    }
}

pub struct Listeners {
    file_listener: Option<FileListenerData>,
    time_listener: Option<Vec<TimeListenerData>>,
    workspace_listener: Option<WorkspaceListenerData>,
    volume_listener: Option<broadcast::Sender<()>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            file_listener: Some(FileListenerData::new()),
            time_listener: Some(Vec::new()),
            workspace_listener: WorkspaceListenerData::new().ok(),
            volume_listener: Some(broadcast::Sender::new(1)),
        }
    }

    pub fn new_workspace_listener(&self) -> Option<broadcast::Receiver<()>> {
        Some(self.workspace_listener.as_ref()?.tx.subscribe())
    }

    pub fn start_all(&mut self) {
        let time_listener = self.time_listener.take();
        let file_listener = self.file_listener.take();
        let workspace_listener = self.workspace_listener.take();
        let volume_listener = self.volume_listener.take();

        // TLDR: thread sorts listeners by interval, waits for the shortest interval sends the message
        // to the listeners whose interval has passed and resets the interval in a loop
        thread::spawn(move || {
            if let Some(mut time_listener) = time_listener {
                if time_listener.is_empty() {
                    return;
                }

                loop {
                    time_listener.sort_by(|a, b| a.interval.cmp(&b.interval));
                    let min_interval = time_listener[0].interval;
                    thread::sleep(std::time::Duration::from_millis(min_interval));
                    time_listener.iter_mut().for_each(|data| {
                        if data.interval <= min_interval {
                            _ = data.tx.send(());
                            data.interval = data.original_interval;
                        } else {
                            data.interval -= min_interval;
                        }
                    });
                }
            }
        });

        thread::spawn(move || {
            if let Some(mut file_listener) = file_listener {
                if file_listener.store.is_empty() {
                    return;
                }

                loop {
                    let mut buffer = [0; 1024];
                    if let Ok(events) = file_listener.inotify.read_events_blocking(&mut buffer) {
                        events.for_each(|event| {
                            // We're always listening to parent changes so unwrap is safe (I hope)
                            let name = event.name.unwrap().to_string_lossy();
                            if let Some(tx) = file_listener.store.get(&*name) {
                                _ = tx.send(());
                            }
                        });
                    }
                }
            }
        });

        thread::spawn(move || {
            if let Some(mut workspace_listener) = workspace_listener {
                if workspace_listener.tx.receiver_count() == 0 {
                    return;
                }

                match &mut workspace_listener.listener {
                    WorkspaceListener::Hyprland(listener) => {
                        let _ = listener.start_listener();
                    }
                    WorkspaceListener::Sway(_) => {}
                }
            }
        });

        thread::spawn(move || {
            if let Some(volume_listener) = volume_listener {
                if volume_listener.receiver_count() == 0 {
                    return;
                }

                let mut mainloop = Mainloop::new().unwrap();
                let mut context = Context::new(&mainloop, "volume-change-listener").unwrap();
                _ = context.connect(None, ContextFlagSet::NOFLAGS, None);
                _ = mainloop.start();
                loop {
                    if context.get_state() == libpulse_binding::context::State::Ready {
                        break;
                    }
                }

                context.set_subscribe_callback(Some(Box::new(move |_, _, _| {
                    _ = volume_listener.send(());
                })));
                context.subscribe(InterestMaskSet::SINK, |_| {});

                let mainloop = Box::new(mainloop);
                let context = Box::new(context);
                Box::leak(context);
                Box::leak(mainloop);
            }
        });
    }

    pub fn new_time_listener(&mut self, interval: u64) -> broadcast::Receiver<()> {
        let (tx, rx) = broadcast::channel(1);

        let data = TimeListenerData {
            tx,
            interval,
            original_interval: interval,
        };

        self.time_listener.as_mut().unwrap().push(data);

        rx
    }

    pub fn new_file_listener(&mut self, path: &Path) -> broadcast::Receiver<()> {
        let (tx, rx) = broadcast::channel(1);

        let file_listener = self.file_listener.as_mut().unwrap();
        file_listener
            .store
            .insert(path.file_name().unwrap().to_string_lossy().into(), tx);
        if let Err(e) = file_listener
            .inotify
            .watches()
            .add(path.parent().unwrap(), WatchMask::MODIFY)
        {
            warn!(
                "Failed to create file change listener for path: {}\n {}",
                path.display(),
                e
            );
        }

        rx
    }

    pub fn new_volume_change_listener(&self) -> broadcast::Receiver<()> {
        self.volume_listener.as_ref().unwrap().subscribe()
    }
}
