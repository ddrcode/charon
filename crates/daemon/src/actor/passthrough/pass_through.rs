use evdev::KeyCode;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::domain::{Actor, ActorState, DomainEvent, Event, HidKeyCode, Mode, Modifiers};

pub use super::PassThroughState;

pub struct PassThrough {
    state: ActorState,
    report: PassThroughState,
    hidg: File,
    mode: Mode,
}

impl PassThrough {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let state = ActorState::new("PassThrough", tx, rx);
        let report = PassThroughState::new();
        let hidg = OpenOptions::new()
            .write(true)
            .open("/dev/hidg0")
            .expect("Failed to open HID gadget device");

        Self {
            state,
            report,
            hidg,
            mode: Mode::PassThrough,
        }
    }

    async fn handle_key_press(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.report.update_on_press(key);
        if self
            .report
            .is(HidKeyCode::KEY_CAPSLOCK, Modifiers::default())
        {
            self.toggle_mode().await;
        } else if self.report.is(HidKeyCode::KEY_Q, Modifiers::LEFT_CTRL) {
            self.send(DomainEvent::Exit).await;
        } else {
            self.send_report();
        }
    }

    fn handle_key_release(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.report.update_on_release(key);
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
        self.send(DomainEvent::ModeChange(self.mode.clone())).await;
    }

    fn send_report(&mut self) {
        if self.mode == Mode::PassThrough {
            let report = self.report.to_report();
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
                self.stop().await;
            }
            e => {
                warn!("Unhandled event: {:?}", e);
            }
        }
    }

    pub fn reset(&mut self) {
        self.report.reset();
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
    async fn tick(&mut self) {
        if let Some(event) = self.recv().await {
            self.handle_event(&event).await;
        }
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
