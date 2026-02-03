// SPDX-License-Identifier: GPL-3.0-or-later
use crate::domain::CharonEvent;
use maiko::{Context, Envelope, Meta};
use std::sync::Arc;
use tracing::error;

use crate::port::HIDDevice;

pub struct KeyWriter<D: HIDDevice> {
    ctx: Context<CharonEvent>,
    device: D,
    prev_sender: Arc<str>,
}

impl<D: HIDDevice> KeyWriter<D> {
    pub fn new(ctx: Context<CharonEvent>, device: D) -> Self {
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

impl<D: HIDDevice> maiko::Actor for KeyWriter<D> {
    type Event = CharonEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        match envelope.event() {
            CharonEvent::HidReport(report) => {
                self.send_report(report, envelope.meta().actor_name());
                self.send_telemetry(envelope.meta()).await?;
            }
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
