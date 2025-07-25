use std::{borrow::Cow, path::PathBuf};

use charon_lib::event::{DomainEvent, Event};
use tokio::task::JoinHandle;
use tracing::debug;

use crate::{
    devices::HIDKeyboard,
    domain::{ActorState, traits::Actor},
};

pub struct KeyWriter {
    state: ActorState,
    device: HIDKeyboard,
    prev_sender: Cow<'static, str>,
}

impl KeyWriter {
    pub fn new(state: ActorState, device_path: &PathBuf) -> Self {
        Self {
            state,
            device: HIDKeyboard::new(device_path),
            prev_sender: "".into(),
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::HidReport(report) => {
                self.send_report(report, &event.sender);
                self.send_telemetry(event).await;
            }
            DomainEvent::Exit => self.stop().await,
            DomainEvent::ModeChange(_) => self.device.reset(),
            _ => {}
        }
    }

    fn send_report(&mut self, report: &[u8; 8], sender: &Cow<'static, str>) {
        if self.prev_sender != *sender {
            self.device.reset();
            self.prev_sender = sender.clone()
        }
        debug!("Writing report to HID controller: {:?}", report);
        self.device.send_report(report);
    }

    async fn send_telemetry(&mut self, event: &Event) {
        if self.state.config().enable_telemetry {
            self.send_raw(Event::with_source_id(
                self.id(),
                DomainEvent::ReportSent(),
                event.source_event_id.unwrap().clone(),
            ))
            .await;
        }
    }
}

#[async_trait::async_trait]
impl Actor for KeyWriter {
    type Init = ();

    fn name() -> &'static str {
        "KeyWriter"
    }

    fn spawn(state: ActorState, (): ()) -> JoinHandle<()> {
        let dev = state.config().hid_keyboard.clone();
        let mut writer = KeyWriter::new(state, &dev);
        tokio::spawn(async move { writer.run().await })
    }

    async fn tick(&mut self) {
        if let Some(event) = self.recv().await {
            self.handle_event(&event).await;
        }
    }

    async fn shutdown(&mut self) {
        self.device.reset();
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
