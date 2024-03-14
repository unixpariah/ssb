use hyprland::event_listener::EventListener;
use std::{cell::RefCell, rc::Rc, sync::Mutex, thread};
use tokio::sync::broadcast;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(&'static str),
}

pub struct Listeners {
    pub workspace_listener: Option<Rc<RefCell<broadcast::Sender<bool>>>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            workspace_listener: None,
        }
    }

    pub fn new_workspace_listener(&mut self) -> broadcast::Receiver<bool> {
        if let Some(workspace_listener) = &self.workspace_listener {
            return workspace_listener.borrow().subscribe();
        }

        let mut listener = EventListener::new();

        let (tx, rx) = broadcast::channel(1);
        let tx = Rc::new(RefCell::new(tx));

        {
            let tx = Rc::clone(&tx);
            listener.add_workspace_destroy_handler(move |_| {
                let _ = tx.borrow().send(true);
            });
        }

        {
            let tx = Rc::clone(&tx);
            listener.add_workspace_change_handler(move |_| {
                let _ = tx.borrow().send(true);
            });
        }

        {
            let tx = Rc::clone(&tx);
            listener.add_active_monitor_change_handler(move |_| {
                let _ = tx.borrow().send(true);
            });
        }

        thread::spawn(move || {
            listener.start_listener().expect("Failed to start listener");
        });

        self.workspace_listener = Some(tx);
        rx
    }
}

pub fn create_time_passed_listener(interval: u64) -> broadcast::Receiver<bool> {
    let (tx, rx) = broadcast::channel(1);

    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(interval));
        let _ = tx.send(true);
    });

    rx
}

static mut HOTWATCH: Mutex<Option<hotwatch::Hotwatch>> = Mutex::new(None);

pub fn create_file_change_listener(path: &'static str) -> broadcast::Receiver<bool> {
    let (tx, rx) = broadcast::channel(1);

    unsafe {
        let mut hotwatch =
            hotwatch::Hotwatch::new_with_custom_delay(std::time::Duration::from_millis(10))
                .expect("Failed to create hotwatch");

        hotwatch
            .watch(path, move |event| {
                if let hotwatch::EventKind::Modify(_) = event.kind {
                    let _ = tx.send(true);
                }
            })
            .expect("Failed to watch file");

        if let Ok(mut ht) = HOTWATCH.lock() {
            *ht = Some(hotwatch);
        }
    }

    rx
}
