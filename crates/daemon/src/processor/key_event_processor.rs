use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use maiko::Meta;
use tracing::{debug, error};

use crate::domain::{HidKeyCode, KeyboardState, traits::Processor};

#[derive(Default)]
pub struct KeyEventProcessor {
    report: KeyboardState,
    events: Vec<DomainEvent>,
}

impl KeyEventProcessor {
    async fn handle_key_press(&mut self, key: &KeyCode, meta: Meta) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_press(key);
        self.send_report(meta).await;
    }

    async fn handle_key_release(&mut self, key: &KeyCode, meta: Meta) {
        let key = match HidKeyCode::try_from(key) {
            Ok(val) => val,
            Err(e) => {
                return error!("{e}");
            }
        };
        self.report.update_on_release(key);
        self.send_report(meta).await;
    }

    async fn send_report(&mut self, _meta: Meta) {
        let report = self.report.to_report();
        let payload = DomainEvent::HidReport(report);
        // let event = Event::with_source_id(self.state.id.clone(), payload, meta);
        // self.events.push(event);
        self.events.push(payload);
    }
}

#[async_trait::async_trait]
impl Processor for KeyEventProcessor {
    async fn process(&mut self, event: DomainEvent, meta: Meta) -> Vec<DomainEvent> {
        debug!("GOT EVENT: {event:?}");
        match &event {
            DomainEvent::KeyPress(key, _) => self.handle_key_press(key, meta).await,
            DomainEvent::KeyRelease(key, _) => self.handle_key_release(key, meta).await,
            _ => self.events.push(event),
        }
        std::mem::take(&mut self.events)
    }
}
