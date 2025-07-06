use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

use crate::domain::{DomainEvent, Event};
use evdev::{Device, EventSummary};
use tracing::{error, info, warn};

pub struct KeyScanner {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    device: Device,
    alive: bool,
}

impl KeyScanner {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let device = Device::open("/dev/input/event5").unwrap();
        KeyScanner {
            device,
            tx,
            rx,
            alive: true,
        }
    }

    pub fn run(&mut self) {
        info!("Starting Key Scanner");

        self.grab();

        while self.alive {
            self.check_messages();
            let key_events: Vec<_> = self.device.fetch_events().unwrap().collect();
            for event in key_events {
                let kos_event = match event.destructure() {
                    EventSummary::Key(_, key, value) => match value {
                        1 | 2 => DomainEvent::KeyPress(key),
                        0 => DomainEvent::KeyRelease(key),
                        other => {
                            warn!("Unhandled key event value: {}", other);
                            continue;
                        }
                    },
                    EventSummary::Synchronization(..) | EventSummary::Misc(..) => continue,
                    e => {
                        warn!("Unhandled device event: {:?}", e);
                        continue;
                    }
                };
                self.send(kos_event);
            }
        }
    }

    fn id() -> &'static str {
        "key_scanner"
    }

    fn send(&mut self, payload: DomainEvent) {
        let event = Event::new(Self::id(), payload);
        self.send_raw(event);
    }

    fn send_raw(&mut self, event: Event) {
        if let Err(_) = self.tx.blocking_send(event) {
            warn!("Can't send messages - channel closed. Quitting.");
            self.alive = false;
        }
    }

    fn check_messages(&mut self) {
        match self.rx.try_recv() {
            Ok(event) => self.handle_event(&event),
            Err(TryRecvError::Disconnected) => {
                warn!("Can't read messages - channel closed. Quitting.");
                self.alive = false;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => {
                info!("Exit event received. Quittig...");
                self.alive = false;
            }
            other => {
                warn!("Unhandled event: {:?}", other);
            }
        }
    }

    fn grab(&mut self) {
        if !self.device.is_grabbed() {
            if let Err(e) = self.device.grab() {
                error!("Couldn't grab the device: {}", e);
            }
        }
    }

    fn ungrab(&mut self) {
        if self.device.is_grabbed() {
            if let Err(e) = self.device.ungrab() {
                error!("Couldn't ungrab the device: {}", e);
            }
        }
    }
}

impl Drop for KeyScanner {
    fn drop(&mut self) {
        self.ungrab();
    }
}
