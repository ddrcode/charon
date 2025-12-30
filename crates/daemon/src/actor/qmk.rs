use std::borrow::Cow;

use async_hid::{AsyncHidRead, DeviceReader, DeviceReaderWriter, DeviceWriter, HidBackend};
use charon_lib::{event::CharonEvent, qmk::QMKEvent};
use futures_lite::StreamExt;
use maiko::{Config, Context, Runtime};
use tokio::{select, time::Sleep};
use tracing::{debug, error, info};

// https://docs.qmk.fm/features/rawhid#basic-configuration
pub const USAGE_PAGE: u16 = 0xFF60;
pub const USAGE_ID: u16 = 0x0061;

use crate::domain::ActorState;

#[allow(dead_code)]
pub struct QMK {
    ctx: Context<CharonEvent>,
    state: ActorState,
    keyboard_alias: Cow<'static, str>,
    device: Option<(DeviceReader, DeviceWriter)>,
}

#[allow(dead_code)]
impl QMK {
    pub fn new(
        ctx: Context<CharonEvent>,
        state: ActorState,
        keyboard_alias: Cow<'static, str>,
    ) -> Self {
        Self {
            ctx,
            state,
            keyboard_alias,
            device: None,
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
                CharonEvent::ModeChange(new_mode)
            }
            QMKEvent::ModeChange(mode) => {
                self.state.set_mode(mode).await;
                CharonEvent::ModeChange(mode)
            }
            e => CharonEvent::QMKEvent(e),
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
    type Event = CharonEvent;

    async fn on_start(&mut self) -> maiko::Result {
        self.device = Self::find_device(self.vendor_id(), self.product_id()).await;
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> maiko::Result<()> {
        if matches!(event, CharonEvent::Exit) {
            self.ctx.stop();
        }
        Ok(())
    }

    async fn tick(&mut self, runtime: &mut Runtime<'_, Self::Event>) -> maiko::Result {
        let timeout = tokio::time::sleep(runtime.config.tick_interval);
        tokio::pin!(timeout);

        select! {
            biased;
            Ok((_n, buf)) = Self::read_buf(self.device.as_mut()) => {
                self.handle_qmk_message(buf).await?;
            }
            Some(ref envelope) = runtime.recv() => {
                runtime.default_handle(self, envelope).await?;
            }
            _ = timeout => {}
        }
        Ok(())
    }
}
