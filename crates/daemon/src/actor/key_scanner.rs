use std::{collections::HashSet, path::PathBuf};

use charon_lib::event::{DomainEvent, Event, Mode};
use tokio::{io::unix::AsyncFd, task::JoinHandle};
use uuid::Uuid;

use crate::{
    devices::evdev::find_input_device,
    domain::{ActorState, traits::Actor},
    util::time::get_delta_since_start,
};
use evdev::{Device, EventSummary, InputEvent};
use tracing::{debug, error, warn};

/// The key actor of Charon, that scans evdev (input device) on Linux side
/// and sends each captured event to the rest of the system.
/// The events are produced regardless the mode (pass-through / in-app),
/// however the mode determines whether input device is grabbed (pass-through)
/// or not (in-app). The intention is that if in pass-through mode
/// the key events should be send only to the host, while when in in-app mode
/// the keyboard is available to Charon device.
pub struct KeyScanner {
    /// Actor's state
    state: ActorState,

    /// System input device (/dev/input)
    device: AsyncFd<Device>,

    /// Keyboard name added to every key event. Uses alias (if defined in config file)
    /// or device name as in /dev/input/by-id/
    /// It allows handling multiple keyboards by the rest of the system and is useful
    /// when multiple instances of KeyScanner are active.
    keyboard_name: String,

    /// Grab/ungrab intention (actual switch happens when all keys are released)
    should_handle_grab: Option<Mode>,

    /// Keeps currently pressed key codes. Used for clean grab/ungrab of input device.
    keyboard_state: HashSet<u16>,
}

impl KeyScanner {
    pub fn new(state: ActorState, device_path: PathBuf, keyboard_name: String) -> Self {
        let device = Device::open(device_path).unwrap();
        let async_dev = AsyncFd::new(device).unwrap();

        KeyScanner {
            state,
            device: async_dev,
            keyboard_name,
            should_handle_grab: None,
            keyboard_state: HashSet::new(),
        }
    }

    async fn handle_device_events(&mut self, key_events: Vec<InputEvent>) {
        for event in key_events {
            let (payload, ts) = match event.destructure() {
                // meaning of value: 0 - key release, 1 - key press, 2 - key repeat
                EventSummary::Key(ev, key, value) => match value {
                    1 | 2 => {
                        self.keyboard_state.insert(key.code());
                        (
                            DomainEvent::KeyPress(key, self.keyboard_name.clone()),
                            ev.timestamp(),
                        )
                    }
                    0 => {
                        self.keyboard_state.remove(&key.code());
                        (
                            DomainEvent::KeyRelease(key, self.keyboard_name.clone()),
                            ev.timestamp(),
                        )
                    }
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
            DomainEvent::ModeChange(mode) => {
                if self.keyboard_state.is_empty() {
                    self.toggle_grabbing(mode);
                } else {
                    self.should_handle_grab = Some(*mode);
                }
            }
            other => {
                debug!("Unhandled event: {:?}", other);
            }
        }
    }

    fn toggle_grabbing(&mut self, mode: &Mode) {
        debug!(
            "Toggling device grabbing: switching to {mode}, keys currently pressed: {:?}",
            self.keyboard_state
        );
        self.should_handle_grab = None;
        match mode {
            Mode::PassThrough => self.grab(),
            Mode::InApp => self.ungrab(),
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
    type Init = String;

    fn name() -> &'static str {
        "KeyScanner"
    }

    fn spawn(state: ActorState, keyboard_name: String) -> JoinHandle<()> {
        let input = &state.config().keyboard;
        let device_path = find_input_device(input).expect("Couldn't find keyboard device");
        let mut scanner = KeyScanner::new(state, device_path, keyboard_name);
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
        self.toggle_grabbing(&self.state.mode().await);
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

                // grab/ungrab only when all keys are released
                if self.should_handle_grab.is_some() && self.keyboard_state.is_empty() {
                    if let Some(mode) = self.should_handle_grab {
                        self.toggle_grabbing(&mode);
                    }
                }
            }
        }
    }
}
