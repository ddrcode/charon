use charon_lib::event::{DomainEvent, Event};
use evdev::KeyCode;
use tracing::error;

use crate::domain::{HidKeyCode, KeyboardState, ProcessorState, traits::Processor};

pub struct KeyEventProcessor {
    state: ProcessorState,
    report: KeyboardState,
    events: Vec<Event>,
}

impl KeyEventProcessor {
    pub fn factory(state: ProcessorState) -> Box<dyn Processor + Send + Sync> {
        Box::new(KeyEventProcessor::new(state))
    }

    pub fn new(state: ProcessorState) -> Self {
        Self {
            report: KeyboardState::new(),
            state,
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
        let event = Event::new(self.state.id.clone(), payload);
        self.events.push(event);
    }
}

#[async_trait::async_trait]
impl Processor for KeyEventProcessor {
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
