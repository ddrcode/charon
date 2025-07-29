use std::collections::VecDeque;

use evdev::{Device, InputEvent};
use tokio::io::unix::AsyncFd;

use crate::port::AsyncInputSource;

pub struct EvdevInputSource {
    device: AsyncFd<Device>,
    pending: VecDeque<InputEvent>,
}

impl EvdevInputSource {
    pub fn new(device: AsyncFd<Device>) -> Self {
        Self {
            device,
            pending: VecDeque::new(),
        }
    }
}

#[async_trait::async_trait]
impl AsyncInputSource for EvdevInputSource {
    async fn next_event(&mut self) -> Option<InputEvent> {
        loop {
            if let Some(ev) = self.pending.pop_front() {
                return Some(ev);
            }
            let mut guard = self.device.readable_mut().await.ok()?;
            let device = guard.get_mut().get_mut();

            match device.fetch_events() {
                Ok(events) => {
                    self.pending.extend(events);
                }
                Err(_) => return None,
            }

            guard.clear_ready();
        }
    }

    fn is_grabbed(&self) -> bool {
        self.device.get_ref().is_grabbed()
    }

    fn grab(&mut self) -> std::io::Result<()> {
        self.device.get_mut().grab()
    }

    fn ungrab(&mut self) -> std::io::Result<()> {
        self.device.get_mut().ungrab()
    }
}
