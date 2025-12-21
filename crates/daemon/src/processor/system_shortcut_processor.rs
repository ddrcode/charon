use charon_lib::event::{DomainEvent, Mode};
use maiko::Meta;
use tracing::{debug, error, info};

use crate::{
    domain::{ProcessorState, traits::Processor},
    util::system::wake_host_on_lan,
};

pub struct SystemShortcutProcessor {
    state: ProcessorState,
    events: Vec<DomainEvent>,
}

impl SystemShortcutProcessor {
    pub fn factory(state: ProcessorState) -> Box<dyn Processor + Send + Sync> {
        Box::new(SystemShortcutProcessor::new(state))
    }

    pub fn new(state: ProcessorState) -> Self {
        Self {
            state,
            events: Vec::new(),
        }
    }

    async fn handle_report(&mut self, report: &[u8; 8], parent_meta: &Meta) -> bool {
        let num: u64 = u64::from_ne_bytes(*report);
        let config = self.state.config();

        if num == u64::from(&config.quit_shortcut) {
            self.send_exit(parent_meta).await;
        } else if num == u64::from(&config.toggle_mode_shortcut) {
            self.toggle_mode(parent_meta).await;
        } else if num == u64::from(&config.awake_host_shortcut) {
            self.wake_up_host();
        } else {
            return self.state.mode().await == Mode::PassThrough;
        }

        self.reset_hid(parent_meta);
        false
    }

    async fn send_exit(&mut self, _parent_meta: &Meta) {
        // let event = Event::with_source_id(self.state.id.clone(), DomainEvent::Exit, parent_meta);
        self.events.push(DomainEvent::Exit);
    }

    async fn toggle_mode(&mut self, _parent_meta: &Meta) {
        let new_mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", new_mode);
        self.state.set_mode(new_mode).await;
        let payload = DomainEvent::ModeChange(new_mode);
        // let event = Event::with_source_id(self.state.id.clone(), payload, parent_id);
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

    fn reset_hid(&mut self, _parent_meta: &Meta) {
        let payload = DomainEvent::HidReport([0; 8]);
        // let event = Event::with_source_id(self.state.id.clone(), payload, parent_meta);
        self.events.push(payload);
    }
}

#[async_trait::async_trait]
impl Processor for SystemShortcutProcessor {
    async fn process(&mut self, event: DomainEvent, meta: Meta) -> Vec<DomainEvent> {
        match &event {
            DomainEvent::HidReport(report) => {
                // if meta.correlation_id().is_some() {
                if self.handle_report(report, &meta).await {
                    self.events.push(event);
                }
                // }
            }
            _ => self.events.push(event),
        }
        std::mem::take(&mut self.events)
    }
}
