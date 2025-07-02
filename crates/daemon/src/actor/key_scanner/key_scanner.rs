use crossbeam_channel::{Receiver, Sender};
use evdev::{Device, EventSummary};

use crate::domain::{DomainEvent, Event};

pub struct KeyScanner {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    device: Device,
}

impl KeyScanner {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let device = Device::open("/dev/input1").unwrap();
        KeyScanner { device, tx, rx }
    }

    pub fn run(&mut self) {
        loop {
            for event in self.device.fetch_events().unwrap() {
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
                self.tx.send(Event::new(kos_event)).unwrap();
            }
        }
    }
}
