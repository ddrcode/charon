use crossbeam_channel::{Receiver, Sender};
use evdev::{Device, EventSummary};

use crate::domain::{Actor, DomainEvent, Event};

pub struct KeyScanner {
    tx: Sender<Event>,
    _rx: Receiver<Event>,
    device: Device,
}

impl KeyScanner {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let device = Device::open("/dev/input1").unwrap();
        KeyScanner {
            device,
            tx,
            _rx: rx,
        }
    }
}

impl Actor for KeyScanner {
    fn run(&mut self) {
        loop {
            let key_events: Vec<_> = self.device.fetch_events().unwrap().collect();
            for event in key_events {
                let kos_event = match event.destructure() {
                    EventSummary::Key(ev, key, 1) => {
                        println!("Key '{:?}' was pressed, got event: {:?}", key, ev);
                        DomainEvent::KeyPress(key)
                    }
                    EventSummary::Key(_, key, 0) => {
                        println!("Key {:?} was released", key);
                        DomainEvent::KeyRelease(key)
                    }
                    e => {
                        println!("got a different event: {:?}", e);
                        DomainEvent::Warning(format!("{:?}", e).into())
                    }
                };
                self.send(kos_event).unwrap();
            }
        }
    }

    fn id() -> &'static str {
        "key_scanner"
    }

    fn sender(&self) -> &Sender<Event> {
        &self.tx
    }
}
