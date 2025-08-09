use std::borrow::Cow;

use async_hid::{AsyncHidRead, Device, DeviceReaderWriter, HidBackend, HidResult};
use async_trait::async_trait;
use charon_lib::event::{DomainEvent, Event, QMKEvent};
use futures_lite::StreamExt;
use tokio::{select, task::JoinHandle};
use tracing::{debug, error, info, warn};

// https://docs.qmk.fm/features/rawhid#basic-configuration
pub const USAGE_PAGE: u16 = 0xFF60;
pub const USAGE_ID: u16 = 0x0061;

use crate::{
    config::InputConfig,
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct QMK {
    state: ActorState,
    keyboard_alias: Cow<'static, str>,
}

impl QMK {
    fn new(state: ActorState, keyboard_alias: Cow<'static, str>) -> Self {
        Self {
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

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }

    async fn handle_qmk_message(&mut self, msg: [u8; 32]) {
        match QMKEvent::try_from(msg) {
            Ok(qmk_event) => {
                debug!("QMK event received: {:?}", qmk_event);
                self.process(DomainEvent::QMKEvent(qmk_event)).await;
            }
            Err(err) => {
                error!("Error processing QMK event: {err}");
            }
        }
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
}

#[async_trait]
impl Actor for QMK {
    type Init = ();

    fn name() -> &'static str {
        "QMK"
    }

    fn spawn(state: ActorState, (): ()) -> Result<JoinHandle<()>, CharonError> {
        let alias = match state.config().keyboard {
            InputConfig::Use(ref keyb) => keyb.clone(),
            ref k => {
                return Err(CharonError::QMKError(format!(
                    "{k:?} - insufficient or wrong configuration to enable QMK actor"
                )));
            }
        };

        let raw_enabled = state.config().keyboard_info().is_some_and(|group| {
            group.raw_hid_enabled && group.vendor_id.is_some() && group.product_id.is_some()
        });

        if !raw_enabled {
            return Err(CharonError::QMKError(
                "vendor_id or product_id is not provided or raw_hid_enabled is false".into(),
            ));
        }

        let mut qmk = QMK::new(state, alias);
        Ok(tokio::spawn(async move { qmk.run().await }))
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;
        let mut device = Self::find_device(self.vendor_id(), self.product_id()).await;

        while self.state().alive {
            select! {
                Some(event) = self.recv() => {
                    self.handle_event(&event).await;
                }
                Ok((_n, buf)) = Self::read_buf(device.as_mut()) => {
                    self.handle_qmk_message(buf).await;
                }
            }
        }

        self.shutdown().await;
    }

    async fn tick(&mut self) {}

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }
}
