// SPDX-License-Identifier: GPL-3.0-or-later
use crate::domain::qmk::QMKEvent;
use async_hid::{AsyncHidRead, DeviceReaderWriter, HidBackend};
use futures_lite::StreamExt;
use tracing::{error, info};

use crate::{
    config::CharonConfig,
    error::CharonError,
    port::{QmkDevice, RawHidDevice},
};

// https://docs.qmk.fm/features/rawhid#basic-configuration
const USAGE_PAGE: u16 = 0xFF60;
const USAGE_ID: u16 = 0x0061;

pub struct QmkAsyncHidDevice {
    device: DeviceReaderWriter,
}

impl QmkAsyncHidDevice {
    pub async fn async_new(config: &CharonConfig) -> Self {
        let vendor_id = Self::vendor_id(config);
        let product_id = Self::product_id(config);
        let device = Self::find_device(vendor_id, product_id)
            .await
            .expect("RawHid device not found");
        Self { device }
    }

    fn vendor_id(config: &CharonConfig) -> u16 {
        config
            .keyboard_info()
            .expect("keyboard info must be provided")
            .vendor_id
            .expect("vendor_id must be provided")
    }

    fn product_id(config: &CharonConfig) -> u16 {
        config
            .keyboard_info()
            .expect("keyboard info must be provided")
            .product_id
            .expect("product_id must be provided")
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
}

impl RawHidDevice for QmkAsyncHidDevice {
    async fn read_packet(&mut self) -> Result<(usize, [u8; 32]), CharonError> {
        let mut buf = [0u8; 32];
        let size = self
            .device
            .read_input_report(&mut buf)
            .await
            .inspect_err(|err| error!("Failed reading raw hid: {err}"))?;
        Ok((size, buf))
    }
}

impl QmkDevice for QmkAsyncHidDevice {
    async fn read_event(&mut self) -> Result<Option<QMKEvent>, CharonError> {
        let (size, msg) = self.read_packet().await?;
        if size == 0 {
            return Ok(None);
        }
        let event = QMKEvent::try_from(msg)
            .inspect_err(|err| error!("Failed creating QMK event from message: {err}"))
            .map_err(|_err| {
                CharonError::QMKError("Failed creating QMKEvent from message".into())
            })?;
        Ok(Some(event))
    }
}
