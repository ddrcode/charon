use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use maiko::Meta;
use tracing::error;

use crate::domain::{HidKeyCode, KeyboardState, traits::Processor};

#[derive(Default)]
pub struct KeyEventProcessor {
    report: KeyboardState,
    events: Vec<DomainEvent>,
}

impl KeyEventProcessor {
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
        let event = DomainEvent::HidReport(report);
        self.events.push(event);
    }
}

#[async_trait::async_trait]
impl Processor for KeyEventProcessor {
    async fn process(&mut self, event: DomainEvent, _meta: Meta) -> Vec<DomainEvent> {
        match &event {
            DomainEvent::KeyPress(key, _) => self.handle_key_press(key).await,
            DomainEvent::KeyRelease(key, _) => self.handle_key_release(key).await,
            _ => self.events.push(event),
        }
        std::mem::take(&mut self.events)
    }
}
