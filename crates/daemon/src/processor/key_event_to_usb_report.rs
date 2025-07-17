use charon_lib::event::{DomainEvent, Event};
use evdev::KeyCode;
use tracing::error;

use crate::domain::{HidKeyCode, KeyboardState, Processor};

pub struct KeyEventToUsbReport {
    id: &'static str,
    report: KeyboardState,
    events: Vec<Event>,
}

impl KeyEventToUsbReport {
    pub fn new() -> Self {
        Self {
            id: "PassThroughProcessor",
            report: KeyboardState::new(),
            events: Vec::new(),
        }
    }

    async fn handle_key_press(&mut self, key: &KeyCode) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_press(key);
        self.send_report().await;
    }

    async fn handle_key_release(&mut self, key: &KeyCode) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_release(key);
        self.send_report().await;
    }

    async fn send_report(&mut self) {
        let report = self.report.to_report();
        let payload = DomainEvent::HidReport(report);
        let event = Event::new(self.id.into(), payload);
        self.events.push(event);
    }
}

#[async_trait::async_trait]
impl Processor for KeyEventToUsbReport {
    async fn process(&mut self, input: Vec<Event>) -> Vec<Event> {
        for event in input.into_iter() {
            match &event.payload {
                DomainEvent::KeyPress(key, _) => self.handle_key_press(key).await,
                DomainEvent::KeyRelease(key, _) => self.handle_key_release(key).await,
                _ => self.events.push(event),
            }
        }
        std::mem::take(&mut self.events)
    }
}
