use std::path::PathBuf;

use charon_lib::domain::{DomainEvent, Event, Mode};
use tokio::{io::unix::AsyncFd, task::JoinHandle};

use crate::{
    domain::{Actor, ActorState},
    utils::keyboard::find_keyboard_device,
};
use evdev::{Device, EventSummary, InputEvent};
use tracing::{error, info, warn};

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
            let kos_event = match event.destructure() {
                EventSummary::Key(_, key, value) => match value {
                    1 | 2 => DomainEvent::KeyPress(key),
                    0 => DomainEvent::KeyRelease(key),
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
            self.send(kos_event).await;
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
}

impl Drop for KeyScanner {
    fn drop(&mut self) {
        self.ungrab();
    }
}

#[async_trait::async_trait]
impl Actor for KeyScanner {
    fn spawn(state: ActorState) -> JoinHandle<()> {
        let device_path = find_keyboard_device().expect("Couldn't find keyboard device");
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
        self.grab();
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

    fn filter(event: &Event) -> bool {
        match event.payload {
            DomainEvent::ModeChange(_) => true,
            _ => false,
        }
    }
}
