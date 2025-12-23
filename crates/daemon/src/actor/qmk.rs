use std::borrow::Cow;

use async_hid::{AsyncHidRead, DeviceReaderWriter, HidBackend};
use charon_lib::{event::DomainEvent, qmk::QMKEvent};
use futures_lite::StreamExt;
use maiko::{Context, Meta};
use tracing::{debug, error, info};

// https://docs.qmk.fm/features/rawhid#basic-configuration
pub const USAGE_PAGE: u16 = 0xFF60;
pub const USAGE_ID: u16 = 0x0061;

use crate::domain::ActorState;

#[allow(dead_code)]
pub struct QMK {
    ctx: Context<DomainEvent>,
    state: ActorState,
    keyboard_alias: Cow<'static, str>,
}

#[allow(dead_code)]
impl QMK {
    pub fn new(
        ctx: Context<DomainEvent>,
        state: ActorState,
        keyboard_alias: Cow<'static, str>,
    ) -> Self {
        Self {
            ctx,
            state,
            keyboard_alias,
        }
    }

    fn vendor_id(&self) -> u16 {
        self.state
            .config()
            .keyboard_info()
            .expect("keyboard info must be provided")
            .vendor_id
            .expect("vendor_id must be provided")
    }

    fn product_id(&self) -> u16 {
        self.state
            .config()
            .keyboard_info()
            .expect("keyboard info must be provided")
            .product_id
            .expect("product_id must be provided")
    }

    async fn handle_qmk_message(&mut self, msg: [u8; 32]) -> maiko::Result {
        match QMKEvent::try_from(msg) {
            Ok(qmk_event) => {
                debug!("QMK event received: {:?}", qmk_event);
                self.process_qmk_event(qmk_event).await?;
            }
            Err(err) => {
                error!("Error processing QMK event: {err}");
            }
        }
        Ok(())
    }

    async fn process_qmk_event(&mut self, qmk_event: QMKEvent) -> maiko::Result {
        let event = match qmk_event {
            QMKEvent::ToggleMode => {
                let new_mode = self.state.mode().await.toggle();
                debug!("Switching mode to {:?}", new_mode);
                self.state.set_mode(new_mode).await;
                DomainEvent::ModeChange(new_mode)
            }
            QMKEvent::ModeChange(mode) => {
                self.state.set_mode(mode).await;
                DomainEvent::ModeChange(mode)
            }
            e => DomainEvent::QMKEvent(e),
        };
        self.ctx.send(event).await
    }

    async fn find_device(vendor_id: u16, product_id: u16) -> Option<DeviceReaderWriter> {
        let dev = HidBackend::default()
            .enumerate()
            .await
            .inspect_err(|err| error!("Error enumerating raw hid devices: {err}"))
            .ok()?
            .find(|info| info.matches(USAGE_PAGE, USAGE_ID, vendor_id, product_id))
            .await
            .inspect(|d| info!("Raw HID Device found: {d:?}"));
        if let Some(dev) = dev
            && let Ok(rw) = dev
                .open()
                .await
                .inspect_err(|err| error!("Couldn't open raw hid device: {err}"))
        {
            return Some(rw);
        }
        None
    }

    async fn read_buf(
        device: Option<&mut DeviceReaderWriter>,
    ) -> async_hid::HidResult<(usize, [u8; 32])> {
        let dev = device.ok_or_else(|| async_hid::HidError::Message("Device not found".into()))?;
        let mut buf = [0u8; 32];
        let size = dev
            .read_input_report(&mut buf)
            .await
            .inspect_err(|err| error!("Failed reading raw hid: {err}"))?;
        Ok((size, buf))
    }

    async fn write_buf(&mut self) {}
}

impl maiko::Actor for QMK {
    type Event = DomainEvent;

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> maiko::Result<()> {
        if matches!(event, DomainEvent::Exit) {
            self.ctx.stop();
        }
        Ok(())
    }

    async fn tick(&mut self) -> maiko::Result<()> {
        // TODO it shuldn't be read on every tick!
        let mut device = Self::find_device(self.vendor_id(), self.product_id()).await;
        if let Ok((_n, buf)) = Self::read_buf(device.as_mut()).await {
            self.handle_qmk_message(buf).await?;
        }
        Ok(())
    }
}
