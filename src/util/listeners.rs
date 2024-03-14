use hyprland::event_listener::EventListener;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};

#[derive(Copy, Clone, Debug)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(&'static str),
}

pub fn create_workspace_listener() -> Receiver<bool> {
    let mut listener = EventListener::new();
    let (tx, rx) = std::sync::mpsc::channel();
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

    rx
}

pub struct Listeners {
    pub workspace_listener: Option<Arc<Mutex<Receiver<bool>>>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            workspace_listener: None,
        }
    }

    pub fn new_workspace_listener(&mut self) -> Arc<Mutex<Receiver<bool>>> {
        if let Some(workspace_listener) = &self.workspace_listener {
            return Arc::clone(&workspace_listener);
        }

        let mut listener = EventListener::new();
        let (tx, rx) = std::sync::mpsc::channel();
        let tx = Rc::new(RefCell::new(tx));
        let rx = Arc::new(Mutex::new(rx));

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

        self.workspace_listener = Some(Arc::clone(&rx));

        rx
    }
}

pub fn create_time_passed_listener(interval: u64) -> Receiver<bool> {
    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(interval));
        let _ = tx.send(true);
    });

    rx
}

static mut HOTWATCH: Mutex<Option<hotwatch::Hotwatch>> = Mutex::new(None);

pub fn create_file_change_listener(path: &'static str) -> Receiver<bool> {
    let (tx, rx) = std::sync::mpsc::channel();

    unsafe {
        let mut hotwatch =
            hotwatch::Hotwatch::new_with_custom_delay(std::time::Duration::from_millis(50))
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
