use crossbeam_channel::{Receiver, Sender};

use crate::domain::Event;

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

    pub fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(event) => {
                    if let Err(e) = self.tx.send(event) {
                        eprintln!("Failed to send event: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                    break;
                }
            }
        }
    }
}
