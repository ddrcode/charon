use crossbeam_channel::{Receiver, Sender};
use evdev::{Device, EventSummary, KeyCode};
use std::{thread, time};

use crate::domain::{Actor, DomainEvent, Event};

pub struct KeyScanner {
    tx: Sender<Event>,
    _rx: Receiver<Event>,
    device: Device,
}

impl KeyScanner {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let device = Device::open("/dev/input/event5").unwrap();
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
                    EventSummary::Synchronization(..) | EventSummary::Misc(..) => {
                        // Sync event isn't important in case of keyboard. Skipping
                        // Misc provides system time. Can be usef for timestamping, eventually
                        continue;
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

    // fn run(&mut self) {
    //     let chars: Vec<u16> = vec![30, 31, 32, 33, 34, 35, 36, 37, 38, 44, 45]; // Example key codes
    //     for c in chars.into_iter() {
    //         let key = KeyCode(c);
    //         self.send(DomainEvent::KeyPress(key)).unwrap();
    //         thread::sleep(time::Duration::from_millis(100));
    //         self.send(DomainEvent::KeyRelease(key)).unwrap();
    //         thread::sleep(time::Duration::from_millis(100));
    //     }
    //     loop {}
    // }

    fn id() -> &'static str {
        "key_scanner"
    }

    fn sender(&self) -> &Sender<Event> {
        &self.tx
    }
}
