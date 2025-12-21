use charon_lib::event::{DomainEvent, Mode};
use maiko::Meta;
use tracing::{debug, error, info};

use crate::{
    domain::{ActorState, traits::Processor},
    util::system::wake_host_on_lan,
};

pub struct SystemShortcutProcessor {
    state: ActorState,
    events: Vec<DomainEvent>,
}

impl SystemShortcutProcessor {
    pub fn new(state: ActorState) -> Self {
        Self {
            state,
            events: Vec::new(),
        }
    }

    async fn handle_report(&mut self, report: &[u8; 8]) -> bool {
        let num: u64 = u64::from_ne_bytes(*report);
        let config = self.state.config();

        if num == u64::from(&config.quit_shortcut) {
            self.send_exit().await;
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

    async fn send_exit(&mut self) {
        self.events.push(DomainEvent::Exit);
    }

    async fn toggle_mode(&mut self) {
        let new_mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", new_mode);
        self.state.set_mode(new_mode).await;
        let payload = DomainEvent::ModeChange(new_mode);
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
        let event = DomainEvent::HidReport([0; 8]);
        self.events.push(event);
    }
}

#[async_trait::async_trait]
impl Processor for SystemShortcutProcessor {
    async fn process(&mut self, event: DomainEvent, _meta: Meta) -> Vec<DomainEvent> {
        match &event {
            DomainEvent::HidReport(report) => {
                if self.handle_report(report).await {
                    self.events.push(event);
                }
            }
            _ => self.events.push(event),
        }
        std::mem::take(&mut self.events)
    }
}
