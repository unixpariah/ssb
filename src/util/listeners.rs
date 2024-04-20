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
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
use swayipc::EventType;
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug)]
pub struct TimeListenerData {
    tx: broadcast::Sender<()>,
    interval: u64,
    original_interval: u64,
}

pub struct FileListenerData {
    store: HashMap<String, broadcast::Sender<()>>,
    inotify: Inotify,
}

pub struct Listeners {
    workspace_listener: Arc<Mutex<Option<WorkspaceListenerData>>>,
    file_listener: Arc<Mutex<Option<FileListenerData>>>,
    time_listener: Arc<Mutex<Vec<TimeListenerData>>>,
    volume_listener: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            file_listener: Arc::new(Mutex::new(None)),
            workspace_listener: Arc::new(Mutex::new(None)),
            time_listener: Arc::new(Mutex::new(Vec::new())),
            volume_listener: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_workspace_listener(
        &mut self,
    ) -> Result<broadcast::Receiver<()>, Box<dyn std::error::Error>> {
        if let Ok(workspace_listener) = self.workspace_listener.lock() {
            if let Some(workspace_listener) = workspace_listener.as_ref() {
                return Ok(workspace_listener.tx.subscribe());
            }
        }

        let (tx, rx) = broadcast::channel(1);

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

        self.workspace_listener =
            Arc::new(Mutex::new(Some(WorkspaceListenerData { tx, listener })));
        Ok(rx)
    }

    pub fn start_all(&mut self) {
        let time_listener = Arc::clone(&self.time_listener);
        let file_listener = Arc::clone(&self.file_listener);
        let workspace_listener = Arc::clone(&self.workspace_listener);
        let volume_listener = Arc::clone(&self.volume_listener);

        // TLDR: thread sorts listeners by interval, waits for the shortest interval sends the message
        // to the listeners whose interval has passed and resets the interval in a loop
        thread::spawn(move || {
            if let Ok(mut time_listener) = time_listener.lock() {
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
            if let Ok(mut file_listener) = file_listener.lock() {
                if file_listener.is_none() {
                    return;
                }

                loop {
                    let mut buffer = [0; 1024];
                    if let Some(file_listener) = file_listener.as_mut() {
                        if let Ok(events) = file_listener.inotify.read_events_blocking(&mut buffer)
                        {
                            events.for_each(|event| {
                                if let Some(tx) = file_listener
                                    .store
                                    // We're always listening to parent changes so unwrap is safe (I hope)
                                    .get(&event.name.unwrap().to_string_lossy().to_string())
                                {
                                    _ = tx.send(());
                                }
                            });
                        }
                    }
                }
            }
        });

        thread::spawn(move || {
            if let Ok(mut workspace_listener) = workspace_listener.lock() {
                if workspace_listener.is_none() {
                    return;
                }

                if let Some(listener) = workspace_listener.as_mut() {
                    match &mut listener.listener {
                        WorkspaceListener::Hyprland(listener) => {
                            let _ = listener.start_listener();
                        }
                        WorkspaceListener::Sway(_) => {}
                    }
                }
            }
        });

        thread::spawn(move || {
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
                if let Ok(volume_listener) = volume_listener.lock() {
                    if volume_listener.is_none() {
                        return;
                    }
                    _ = volume_listener.clone().unwrap().send(());
                }
            })));
            context.subscribe(InterestMaskSet::SINK, |_| {});

            let mainloop = Box::new(mainloop);
            let context = Box::new(context);
            Box::leak(context);
            Box::leak(mainloop);
        });
    }

    pub fn new_time_listener(&mut self, interval: u64) -> broadcast::Receiver<()> {
        let (tx, rx) = broadcast::channel(1);

        let data = TimeListenerData {
            tx,
            interval,
            original_interval: interval,
        };

        let time_passed_listener = &self.time_listener;
        if let Ok(mut time_passed_listener) = time_passed_listener.lock() {
            time_passed_listener.push(data);
        }

        rx
    }

    pub fn new_file_listener(&mut self, path: &Path) -> broadcast::Receiver<()> {
        let (tx, rx) = broadcast::channel(1);

        if let Ok(mut file_listener) = self.file_listener.lock() {
            if file_listener.is_none() {
                let mut store = HashMap::new();
                store.insert(path.file_name().unwrap().to_string_lossy().to_string(), tx);
                *file_listener = Some(FileListenerData {
                    store,
                    inotify: Inotify::init().expect("Failed to setup inotify"),
                });
            } else {
                file_listener
                    .as_mut()
                    .unwrap()
                    .store
                    .insert(path.file_name().unwrap().to_string_lossy().to_string(), tx);
            }

            if let Err(e) = file_listener
                .as_mut()
                .unwrap()
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
        }

        rx
    }

    pub fn new_volume_change_listener(&mut self) -> broadcast::Receiver<()> {
        if let Ok(volume_change_listener) = self.volume_listener.lock() {
            if let Some(volume_change_listener) = volume_change_listener.as_ref() {
                return volume_change_listener.subscribe();
            }
        }

        let (tx, rx) = broadcast::channel(1);
        self.volume_listener = Arc::new(Mutex::new(Some(tx)));

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_time_passed_listener() {
        let mut listeners = Listeners::new();
        assert!(listeners.time_listener.lock().unwrap().is_empty());
        let mut listener = listeners.new_time_listener(1000);
        assert!(!listeners.time_listener.lock().unwrap().is_empty());

        let result = listener.try_recv();
        assert!(result.is_err());

        listeners.start_all();
        let result = listener.recv().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_file_change_listener() {
        let mut listeners = Listeners::new();
        assert!(listeners.file_listener.lock().unwrap().is_none());
        _ = listeners.new_file_listener(&PathBuf::from("/tmp"));
        assert!(listeners.file_listener.lock().unwrap().is_some());
    }

    #[tokio::test]
    async fn test_new_volume_change_listener() {
        let mut listeners = Listeners::new();
        assert!(listeners.volume_listener.lock().unwrap().is_none());
        _ = listeners.new_volume_change_listener();
        assert!(listeners.volume_listener.lock().unwrap().is_some());
    }
}
