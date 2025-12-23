use charon_lib::event::CharonEvent;
use maiko::{Context, Meta};
use std::sync::Arc;
use tracing::{debug, error};

use crate::port::HIDDevice;

pub struct KeyWriter {
    ctx: Context<CharonEvent>,
    device: Box<dyn HIDDevice + Send + Sync>,
    prev_sender: Arc<str>,
}

impl KeyWriter {
    pub fn new(ctx: Context<CharonEvent>, device: Box<dyn HIDDevice + Send + Sync>) -> Self {
        Self {
            ctx,
            device,
            prev_sender: "".into(),
        }
    }

    fn send_report(&mut self, report: &[u8; 8], sender: &str) {
        if self.prev_sender.as_ref() != sender {
            self.reset();
            self.prev_sender = Arc::from(sender);
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

    #[inline]
    async fn send_telemetry(&mut self, meta: &Meta) -> maiko::Result<()> {
        // if self.state.config().enable_telemetry {
        if meta.correlation_id().is_some() {
            self.ctx
                .send_child_event(CharonEvent::ReportSent, meta)
                .await?;
        }
        // }
        Ok(())
    }
}

impl maiko::Actor for KeyWriter {
    type Event = CharonEvent;

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> maiko::Result<()> {
        match event {
            CharonEvent::HidReport(report) => {
                self.send_report(report, meta.actor_name());
                self.send_telemetry(meta).await?;
            }
            CharonEvent::Exit => self.ctx.stop(),
            CharonEvent::ModeChange(_) => self.reset(),
            _ => {}
        }
        Ok(())
    }

    async fn on_shutdown(&mut self) -> maiko::Result<()> {
        self.reset();
        Ok(())
    }
}
