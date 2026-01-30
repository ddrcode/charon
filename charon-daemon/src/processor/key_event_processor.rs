use std::pin::Pin;

use crate::domain::CharonEvent;
use evdev::KeyCode;
use maiko::Meta;
use tracing::error;

use crate::domain::{HidKeyCode, KeyboardState, traits::Processor};

#[derive(Default)]
pub struct KeyEventProcessor {
    report: KeyboardState,
    events: Vec<CharonEvent>,
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
        let event = CharonEvent::HidReport(report);
        self.events.push(event);
    }
}

impl Processor for KeyEventProcessor {
    fn process<'a, 'b>(
        &'a mut self,
        event: CharonEvent,
        _meta: Meta,
    ) -> Pin<Box<dyn Future<Output = Vec<CharonEvent>> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(async move {
            match &event {
                CharonEvent::KeyPress(key, _) => self.handle_key_press(key).await,
                CharonEvent::KeyRelease(key, _) => self.handle_key_release(key).await,
                _ => self.events.push(event),
            }
            std::mem::take(&mut self.events)
        })
    }
}
