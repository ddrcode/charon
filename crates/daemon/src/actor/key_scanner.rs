use std::collections::HashSet;

use charon_lib::event::{CharonEvent, Mode};
use maiko::{Context, Envelope, StepAction};

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

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> maiko::Result<()> {
        match envelope.event() {
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

    async fn step(&mut self) -> maiko::Result<StepAction> {
        while let Some(event) = self.input.next_event().await {
            self.handle_device_event(event).await?;

            // grab/ungrab only when all keys are released
            if self.should_handle_grab.is_some() && self.keyboard_state.is_empty() {
                if let Some(mode) = self.should_handle_grab {
                    self.toggle_grabbing(&mode);
                }
            }
        }
        Ok(StepAction::Yield)
    }

    fn on_error(&self, error: maiko::Error) -> maiko::Result<()> {
        error!("Error occured: {:?}", error);
        Err(error)
    }
}
