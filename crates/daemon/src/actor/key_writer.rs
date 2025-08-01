use charon_lib::event::{DomainEvent, Event};
use std::borrow::Cow;
use tokio::task::JoinHandle;
use tracing::{debug, error};

use crate::{
    adapter::HIDDeviceUnix,
    domain::{ActorState, traits::Actor},
    error::CharonError,
    port::HIDDevice,
};

pub struct KeyWriter {
    state: ActorState,
    device: Box<dyn HIDDevice + Send + Sync>,
    prev_sender: Cow<'static, str>,
}

impl KeyWriter {
    pub fn new(state: ActorState, device: Box<dyn HIDDevice + Send + Sync>) -> Self {
        Self {
            state,
            device,
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
            DomainEvent::ModeChange(_) => self.reset(),
            _ => {}
        }
    }

    fn send_report(&mut self, report: &[u8; 8], sender: &Cow<'static, str>) {
        if self.prev_sender != *sender {
            self.reset();
            self.prev_sender = sender.clone()
        }
        debug!("Writing report to HID controller: {:?}", report);
        if let Err(err) = self.device.send_report(report) {
            error!("Error while sending HID report: {err}");
        }
    }

    fn reset(&mut self) {
        if let Err(err) = self.device.reset() {
            error!("Error reseting HID device: {err}");
        }
    }

    async fn send_telemetry(&mut self, event: &Event) {
        if self.state.config().enable_telemetry {
            if let Some(source_id) = event.source_event_id {
                self.send_raw(Event::with_source_id(
                    self.id(),
                    DomainEvent::ReportSent(),
                    source_id,
                ))
                .await;
            }
        }
    }
}

#[async_trait::async_trait]
impl Actor for KeyWriter {
    type Init = ();

    fn name() -> &'static str {
        "KeyWriter"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let dev_path = state.config().hid_keyboard.clone();
        let dev = HIDDeviceUnix::new(&dev_path);
        let mut writer = KeyWriter::new(state, Box::new(dev));
        Ok(tokio::spawn(async move { writer.run().await }))
    }

    async fn tick(&mut self) {
        if let Some(event) = self.recv().await {
            self.handle_event(&event).await;
        }
    }

    async fn shutdown(&mut self) {
        self.reset();
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
