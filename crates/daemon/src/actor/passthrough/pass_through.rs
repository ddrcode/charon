use crossbeam_channel::{Receiver, Sender};
use evdev::KeyCode;

use crate::domain::{Actor, DomainEvent, Event, HidKeyCode};

pub use super::PassThroughState;

pub struct PassThrough {
    state: PassThroughState,
    tx: Sender<Event>,
    rx: Receiver<Event>,
}

impl PassThrough {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let state = PassThroughState::new();
        Self { tx, rx, state }
    }

    fn handle_key_press(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_press(key);
    }

    fn handle_key_release(&mut self, key: &KeyCode) {
        let key = HidKeyCode::try_from(key).unwrap();
        self.state.update_on_release(key);
    }

    fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::KeyPress(key) => {
                self.handle_key_press(key);
            }
            DomainEvent::KeyRelease(key) => {
                self.handle_key_release(key);
            }
            e => {
                println!("Unhandled event: {:?}", e);
            }
        }
    }
}

impl Actor for PassThrough {
    fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(event) => {
                    self.handle_event(&event);
                }
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                    break;
                }
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
