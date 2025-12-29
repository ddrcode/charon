use std::collections::HashSet;

use charon_lib::event::{CharonEvent, Mode};
use maiko::{Context, Runtime};
use tokio::select;

use crate::{domain::ActorState, port::EventDevice};
use evdev::{EventSummary, InputEvent};
use tracing::{debug, error, warn};

/// The key actor of Charon, that scans evdev (input device) on Linux side
/// and sends each captured event to the rest of the system.
/// The events are produced regardless the mode (pass-through / in-app),
/// however the mode determines whether input device is grabbed (pass-through)
/// or not (in-app). The intention is that if in pass-through mode
/// the key events should be send only to the host, while when in in-app mode
/// the keyboard is available to Charon device.
pub struct KeyScanner {
    ctx: Context<CharonEvent>,

    /// Actor's state
    state: ActorState,

    /// System input device (/dev/input)
    input: Box<dyn EventDevice>,

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
    pub fn new(
        ctx: Context<CharonEvent>,
        state: ActorState,
        input: Box<dyn EventDevice>,
        keyboard_name: String,
    ) -> Self {
        KeyScanner {
            ctx,
            state,
            input,
            keyboard_name,
            should_handle_grab: None,
            keyboard_state: HashSet::new(),
        }
    }

    async fn handle_device_event(&mut self, key_event: InputEvent) -> maiko::Result<()> {
        let payload = match key_event.destructure() {
            // meaning of value: 0 - key release, 1 - key press, 2 - key repeat
            EventSummary::Key(_, key, value) => match value {
                1 | 2 => {
                    self.keyboard_state.insert(key.code());
                    CharonEvent::KeyPress(key, self.keyboard_name.clone())
                }
                0 => {
                    self.keyboard_state.remove(&key.code());
                    CharonEvent::KeyRelease(key, self.keyboard_name.clone())
                }
                other => {
                    warn!("Unhandled key event value: {}", other);
                    return Ok(());
                }
            },
            EventSummary::Synchronization(..) | EventSummary::Misc(..) => return Ok(()),
            e => {
                warn!("Unhandled device event: {:?}", e);
                return Ok(());
            }
        };

        self.ctx.send(payload).await
    }

    fn toggle_grabbing(&mut self, mode: &Mode) {
        debug!(
            "Toggling device grabbing: switching to {mode}, keys currently pressed: {:?}",
            self.keyboard_state
        );
        if self.keyboard_state.is_empty() {
            self.should_handle_grab = None;
            match mode {
                Mode::PassThrough => self.grab(),
                Mode::InApp => self.ungrab(),
            }
        } else {
            self.should_handle_grab = Some(*mode);
        }
    }

    fn grab(&mut self) {
        if !self.input.is_grabbed() {
            if let Err(e) = self.input.grab() {
                error!("Couldn't grab the device: {}", e);
            }
        }
    }

    fn ungrab(&mut self) {
        if self.input.is_grabbed()
            && let Err(e) = self.input.ungrab()
        {
            error!("Couldn't ungrab the device: {}", e);
        }
    }
}

impl Drop for KeyScanner {
    fn drop(&mut self) {
        self.ungrab();
    }
}

impl maiko::Actor for KeyScanner {
    type Event = CharonEvent;

    async fn on_start(&mut self) -> maiko::Result<()> {
        self.toggle_grabbing(&self.state.mode().await);
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> maiko::Result<()> {
        match event {
            CharonEvent::Exit => {
                self.ctx.stop();
            }
            CharonEvent::ModeChange(mode) => {
                self.toggle_grabbing(mode);
            }
            other => {
                debug!("Unhandled event: {:?}", other);
            }
        }
        Ok(())
    }

    async fn tick(&mut self, runtime: &mut Runtime<'_, Self::Event>) -> maiko::Result {
        select! {
            Some(ref envelope) = runtime.recv() => {
                runtime.default_handle(self, envelope).await?;
            }
            Some(device_event) = self.input.next_event() => {
                self.handle_device_event(device_event).await?;

                // grab/ungrab only when all keys are released
                if self.should_handle_grab.is_some() && self.keyboard_state.is_empty() {
                    if let Some(mode) = self.should_handle_grab {
                        self.toggle_grabbing(&mode);
                    }
                }
            }
        }
        Ok(())
    }

    fn on_error(&self, error: maiko::Error) -> maiko::Result<()> {
        error!("Error occured: {:?}", error);
        Err(error)
    }
}

// -------------------------------------------------------------------
// TESTS

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        adapter::mock::EventDeviceMock,
        config::CharonConfig,
        util::test::{macros::*, switch_mode},
    };

    use super::*;
    use evdev::KeyCode;
    use tokio::{
        self,
        sync::{
            RwLock,
            mpsc::{self, Sender},
        },
    };

    fn create_scanner(
        scanner_tx: Sender<Event>,
        input: EventDeviceMock,
    ) -> (KeyScanner, Sender<Event>) {
        let (broker_tx, scanner_rx) = mpsc::channel::<Event>(16);
        let state = ActorState::new(
            "test-scanner".into(),
            Arc::new(RwLock::new(Mode::PassThrough)),
            scanner_tx,
            scanner_rx,
            CharonConfig::default(),
            Vec::new(),
        );
        let scanner = KeyScanner::new(state, Box::new(input), "test-keyboard".into());
        (scanner, broker_tx)
    }

    #[tokio::test]
    async fn test_delayed_ungrab_after_key_release() {
        let input = EventDeviceMock::default();
        let state = input.state().clone();
        let (scanner_tx, mut broker_rx) = mpsc::channel::<Event>(16);
        let (mut scanner, broker_tx) = create_scanner(scanner_tx, input);
        scanner.init().await;

        // Initial state - Charon is in pass-through mode, so device is grabbed (on init)
        with_lock!(state, |lock| {
            assert!(lock.grabbed, "Device should be grabbed after init");
            assert_eq!(lock.grab_calls, 1);
        });

        {
            state.lock().await.simulate_key_press(KeyCode::KEY_A);
        }

        scanner.tick().await;

        // Mode is switched to in-app, but device remains grabbed until all keys
        // are released
        switch_mode(&broker_tx, Mode::InApp).await;
        scanner.tick().await;
        assert_event_matches!(broker_rx, CharonEvent::KeyPress(KeyCode::KEY_A, ..));

        with_lock!(state, |lock| {
            assert!(
                lock.grabbed,
                "Device should remain grabbed until all keys are released"
            );
            assert_eq!(lock.ungrab_calls, 0);
        });

        // Releasing the key should materialize the ungrab request
        {
            state.lock().await.simulate_key_release(KeyCode::KEY_A);
        }

        scanner.tick().await;
        assert_event_matches!(broker_rx, CharonEvent::KeyRelease(KeyCode::KEY_A, ..));

        with_lock!(state, |lock| {
            assert!(!lock.grabbed);
            assert_eq!(lock.ungrab_calls, 1);
        });

        // Grabbing / ungrabbing shouldn't be called on 2nd switch to the same mode
        switch_mode(&broker_tx, Mode::InApp).await;
        scanner.tick().await;

        with_lock!(state, |lock| {
            assert!(!lock.grabbed);
            assert_eq!(lock.ungrab_calls, 1);
        });

        // Finally, grabbing (when no key events pending) should happen
        // immediately on mode change
        switch_mode(&broker_tx, Mode::PassThrough).await;
        scanner.tick().await;

        with_lock!(state, |lock| {
            assert!(lock.grabbed);
            assert_eq!(lock.ungrab_calls, 1);
            assert_eq!(lock.grab_calls, 2);
        });
    }
}
