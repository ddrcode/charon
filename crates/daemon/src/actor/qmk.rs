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
    domain::{ActorState, traits::Actor},
    error::CharonError,
};

pub struct QMK {
    state: ActorState,
}

impl QMK {
    fn new(state: ActorState) -> Self {
        Self { state }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => self.stop().await,
            _ => {}
        }
    }

    async fn handle_qmk_message(&mut self, msg: [u8; 32]) {
        let qmk_event = match msg[0] {
            1 => QMKEvent::LayerChange(msg[1]),
            n => {
                warn!("Unrecognized message id: {n}");
                return;
            }
        };

        debug!("QMK event received: {:?}", qmk_event);
        self.send(DomainEvent::QMKEvent(qmk_event)).await;
    }

    async fn find_device() -> Option<DeviceReaderWriter> {
        let dev = HidBackend::default()
            .enumerate()
            .await
            .inspect_err(|err| error!("Error enumerating raw hid devices: {err}"))
            .ok()?
            .find(|info| info.usage_page == USAGE_PAGE && info.usage_id == USAGE_ID)
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
        let mut qmk = QMK::new(state);
        Ok(tokio::spawn(async move { qmk.run().await }))
    }

    async fn run(&mut self) {
        info!("Starting actor: {}", self.id());
        self.init().await;
        let mut device = Self::find_device().await;

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
