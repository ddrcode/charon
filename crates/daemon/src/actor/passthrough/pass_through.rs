use evdev::KeyCode;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info, warn};

use crate::domain::{Actor, DomainEvent, Event, HidKeyCode};

pub use super::PassThroughState;

pub struct PassThrough {
    state: PassThroughState,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    hidg: File,
    alive: bool,
}

impl PassThrough {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let state = PassThroughState::new();
        let hidg = OpenOptions::new()
            .write(true)
            .open("/dev/hidg0")
            .expect("Failed to open HID gadget device");
        Self {
            tx,
            rx,
            state,
            hidg,
            alive: true,
        }
    }

    fn handle_key_press(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_press(key);
        self.send_report();
    }

    fn handle_key_release(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_release(key);
        self.send_report();
    }

    fn send_report(&mut self) {
        let report = self.state.to_report();
        if let Err(e) = self.hidg.write_all(&report) {
            error!("Failed to write HID report: {}", e);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key) => {
                self.handle_key_press(key);
            }
            DomainEvent::KeyRelease(key) => {
                self.handle_key_release(key);
            }
            DomainEvent::Exit => {
                info!("Exit event received. Quitting...");
                self.alive = false;
            }
            e => {
                warn!("Unhandled event: {:?}", e);
            }
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.send_report();
    }
}

impl Drop for PassThrough {
    fn drop(&mut self) {
        self.reset();
    }
}

#[async_trait::async_trait]
impl Actor for PassThrough {
    async fn run(&mut self) {
        while self.alive {
            if let Some(event) = self.rx.recv().await {
                self.handle_event(&event);
            }
        }
    }

    fn id() -> &'static str {
        "pass-through"
    }

    fn sender(&self) -> &Sender<Event> {
        &self.tx
    }
}
