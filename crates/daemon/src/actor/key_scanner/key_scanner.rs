use std::path::PathBuf;

use charon_lib::event::{DomainEvent, Event, Mode};
use tokio::{io::unix::AsyncFd, task::JoinHandle};
use uuid::Uuid;

use super::find_input_device;
use crate::{
    domain::{Actor, ActorState},
    util::time::get_delta_since_start,
};
use evdev::{Device, EventSummary, InputEvent};
use tracing::{error, warn};

pub struct KeyScanner {
    state: ActorState,
    device: AsyncFd<Device>,
}

impl KeyScanner {
    pub fn new(state: ActorState, device_path: PathBuf) -> Self {
        let device = Device::open(device_path).unwrap();
        let async_dev = AsyncFd::new(device).unwrap();

        KeyScanner {
            state,
            device: async_dev,
        }
    }

    async fn handle_device_events(&mut self, key_events: Vec<InputEvent>) {
        for event in key_events {
            let (payload, ts) = match event.destructure() {
                EventSummary::Key(ev, key, value) => match value {
                    1 | 2 => (DomainEvent::KeyPress(key), ev.timestamp()),
                    0 => (DomainEvent::KeyRelease(key), ev.timestamp()),
                    other => {
                        warn!("Unhandled key event value: {}", other);
                        continue;
                    }
                },
                EventSummary::Synchronization(..) | EventSummary::Misc(..) => continue,
                e => {
                    warn!("Unhandled device event: {:?}", e);
                    continue;
                }
            };

            let charon_event = Event::with_time(self.id(), payload, ts);
            self.send_telemetry(&charon_event.id).await;
            self.send_raw(charon_event).await;
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        match &event.payload {
            DomainEvent::Exit => {
                self.stop().await;
            }
            DomainEvent::ModeChange(mode) => self.switch_mode(mode),
            other => {
                warn!("Unhandled event: {:?}", other);
            }
        }
    }

    fn switch_mode(&mut self, mode: &Mode) {
        match mode {
            Mode::PassThrough => self.grab(),
            Mode::InApp => self.ungrab(),
            _ => todo!(),
        }
    }

    fn grab(&mut self) {
        if !self.device.get_ref().is_grabbed() {
            if let Err(e) = self.device.get_mut().grab() {
                error!("Couldn't grab the device: {}", e);
            }
        }
    }

    fn ungrab(&mut self) {
        if self.device.get_ref().is_grabbed() {
            if let Err(e) = self.device.get_mut().ungrab() {
                error!("Couldn't ungrab the device: {}", e);
            }
        }
    }

    async fn send_telemetry(&mut self, source_id: &Uuid) {
        if self.state.config().enable_telemetry {
            self.send_raw(Event::with_source_id(
                self.id(),
                DomainEvent::KeySent(get_delta_since_start(self.state.start_time())),
                source_id.clone(),
            ))
            .await;
        }
    }
}

impl Drop for KeyScanner {
    fn drop(&mut self) {
        self.ungrab();
    }
}

#[async_trait::async_trait]
impl Actor for KeyScanner {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let input = &state.config().keyboard;
        let device_path = find_input_device(input).expect("Couldn't find keyboard device");
        let mut scanner = KeyScanner::new(state, device_path);
        tokio::task::spawn(async move {
            scanner.run().await;
        })
    }

    fn state(&self) -> &ActorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ActorState {
        &mut self.state
    }

    async fn init(&mut self) {
        self.switch_mode(&self.state.mode().await);
    }

    async fn tick(&mut self) {
        tokio::select! {
            Some(event) = self.state.receiver.recv() => {
                self.handle_event(&event).await;
            }
            ready = self.device.readable_mut() => {
                let events = {
                    let mut guard = ready.unwrap();
                    let device = guard.get_mut().get_mut();
                    device.fetch_events().unwrap().collect::<Vec<_>>()
                };
                self.handle_device_events(events).await;
            }
        }
    }
}
