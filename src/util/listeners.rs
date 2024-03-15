use hotwatch::Hotwatch;
use hyprland::event_listener::EventListener;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::sync::broadcast;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(&'static str),
}

pub struct ListenerData {
    tx: broadcast::Sender<bool>,
    interval: u64,
    original_interval: u64,
}

pub struct Listeners {
    pub workspace_listener: Option<broadcast::Sender<bool>>,
    pub file_change_listener: Hotwatch,
    pub time_passed_listener: Arc<Mutex<Vec<ListenerData>>>,
}

impl Listeners {
    pub fn new() -> Self {
        let file_change_listener = Hotwatch::new_with_custom_delay(Duration::from_millis(50))
            .expect("Failed to create hotwatch");

        Self {
            workspace_listener: None,
            file_change_listener,
            time_passed_listener: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_workspace_listener(&mut self) -> broadcast::Receiver<bool> {
        if let Some(workspace_listener) = &self.workspace_listener {
            return workspace_listener.subscribe();
        }

        let mut listener = EventListener::new();

        let (tx, rx) = broadcast::channel(1);

        {
            let tx = tx.clone();
            listener.add_workspace_destroy_handler(move |_| {
                let _ = tx.send(true);
            });
        }

        {
            let tx = tx.clone();
            listener.add_workspace_change_handler(move |_| {
                let _ = tx.send(true);
            });
        }

        {
            let tx = tx.clone();
            listener.add_active_monitor_change_handler(move |_| {
                let _ = tx.send(true);
            });
        }

        thread::spawn(move || {
            listener.start_listener().expect("Failed to start listener");
        });

        self.workspace_listener = Some(tx);
        rx
    }

    pub fn start_time_passed_listeners(&mut self) {
        let time_passed_listener = Arc::clone(&self.time_passed_listener);

        if time_passed_listener.lock().unwrap().is_empty() {
            return;
        }

        thread::spawn(move || loop {
            if let Ok(mut time_passed_listener) = time_passed_listener.lock() {
                time_passed_listener.sort_unstable_by(|a, b| a.interval.cmp(&b.interval));
                let min_interval = time_passed_listener[0].interval;
                thread::sleep(std::time::Duration::from_millis(min_interval));
                for data in time_passed_listener.iter_mut() {
                    if data.interval <= min_interval {
                        let _ = data.tx.send(true);
                        data.interval = data.original_interval;
                    } else {
                        data.interval -= min_interval;
                    }
                }
            }
        });
    }

    pub fn new_time_passed_listener(&mut self, interval: u64) -> broadcast::Receiver<bool> {
        let (tx, rx) = broadcast::channel(1);

        let data = ListenerData {
            tx,
            interval,
            original_interval: interval,
        };

        let time_passed_listener = Arc::clone(&self.time_passed_listener);
        if let Ok(mut time_passed_listener) = time_passed_listener.try_lock() {
            time_passed_listener.push(data);
        }

        rx
    }

    pub fn new_file_change_listener(&mut self, path: &'static str) -> broadcast::Receiver<bool> {
        let (tx, rx) = broadcast::channel(1);

        self.file_change_listener
            .watch(path, move |event| {
                if let hotwatch::EventKind::Modify(_) = event.kind {
                    let _ = tx.send(true);
                }
            })
            .expect("Failed to watch file");

        rx
    }
}
