// SPDX-License-Identifier: GPL-3.0-or-later
use crate::domain::{CharonEvent, Mode, traits::ProcessorFuture};
use maiko::{Context, Meta};
use tracing::{debug, error, info};

use crate::{
    domain::{ActorState, traits::Processor},
    util::system::wake_host_on_lan,
};

pub struct SystemShortcutProcessor {
    state: ActorState,
    events: Vec<CharonEvent>,
    ctx: Context<CharonEvent>,
}

impl SystemShortcutProcessor {
    pub fn new(ctx: Context<CharonEvent>, state: ActorState) -> Self {
        Self {
            ctx,
            state,
            events: Vec::new(),
        }
    }

    async fn handle_report(&mut self, report: &[u8; 8]) -> bool {
        let num: u64 = u64::from_ne_bytes(*report);
        let config = self.state.config();

        if num == u64::from(&config.quit_shortcut) {
            self.ctx.stop();
        } else if num == u64::from(&config.toggle_mode_shortcut) {
            self.toggle_mode().await;
        } else if num == u64::from(&config.awake_host_shortcut) {
            self.wake_up_host();
        } else {
            return self.state.mode().await == Mode::PassThrough;
        }

        self.reset_hid();
        false
    }

    async fn toggle_mode(&mut self) {
        let new_mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", new_mode);
        self.state.set_mode(new_mode).await;
        let payload = CharonEvent::ModeChange(new_mode);
        self.events.push(payload);
    }

    fn wake_up_host(&self) {
        let config = self.state.config();
        if let Some(ref mac) = config.host_mac_address {
            let mac_addr: [u8; 6] = mac.as_slice().try_into().expect("Incorrect MAC format");
            match wake_host_on_lan(&mac_addr) {
                Ok(_) => info!("Magic packet sent"),
                Err(e) => error!("Error while sendimg magic packet: {e}"),
            }
        }
    }

    fn reset_hid(&mut self) {
        let event = CharonEvent::HidReport([0; 8]);
        self.events.push(event);
    }
}

impl Processor for SystemShortcutProcessor {
    fn process<'a>(&'a mut self, event: CharonEvent, _meta: Meta) -> ProcessorFuture<'a> {
        Box::pin(async move {
            match &event {
                CharonEvent::HidReport(report) => {
                    if self.handle_report(report).await {
                        self.events.push(event);
                    }
                }
                _ => self.events.push(event),
            }
            std::mem::take(&mut self.events)
        })
    }
}
