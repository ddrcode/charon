use evdev::KeyCode;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::domain::{Actor, DomainEvent, Event, HidKeyCode, Mode, Modifiers};

pub use super::PassThroughState;

pub struct PassThrough {
    state: PassThroughState,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    hidg: File,
    alive: bool,
    mode: Mode,
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
            mode: Mode::PassThrough,
        }
    }

    async fn handle_key_press(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_press(key);
        if self
            .state
            .is(HidKeyCode::KEY_CAPSLOCK, Modifiers::default())
        {
            self.toggle_mode().await;
        } else if self.state.is(HidKeyCode::KEY_Q, Modifiers::LEFT_CTRL) {
            self.send(DomainEvent::Exit).await.unwrap();
        } else {
            self.send_report();
        }
    }

    fn handle_key_release(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_release(key);
        self.send_report();
    }

    async fn toggle_mode(&mut self) {
        self.reset();
        self.mode = if self.mode == Mode::PassThrough {
            Mode::InApp
        } else {
            Mode::PassThrough
        };
        debug!("Switching mode to {:?}", self.mode);
        self.send(DomainEvent::ModeChange(self.mode.clone()))
            .await
            .unwrap();
    }

    fn send_report(&mut self) {
        if self.mode == Mode::PassThrough {
            let report = self.state.to_report();
            if let Err(e) = self.hidg.write_all(&report) {
                error!("Failed to write HID report: {}", e);
            }
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key) => {
                self.handle_key_press(key).await;
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
        info!("Starting pass-through service");
        while self.alive {
            if let Some(event) = self.rx.recv().await {
                self.handle_event(&event).await;
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
