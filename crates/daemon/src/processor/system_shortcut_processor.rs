use charon_lib::event::{DomainEvent, Event};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    domain::{ProcessorState, traits::Processor},
    util::{system::wake_host_on_lan, time::get_delta_since_start},
};

pub struct SystemShortcutProcessor {
    state: ProcessorState,
    events: Vec<Event>,
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

    async fn handle_report(&mut self, report: &[u8; 8], parent_id: Uuid) -> bool {
        let num: u64 = u64::from_ne_bytes(*report);
        let config = self.state.config();

        if num == u64::from(&config.quit_shortcut) {
            self.send_exit(parent_id).await;
        } else if num == u64::from(&config.toggle_mode_shortcut) {
            self.toggle_mode(parent_id).await;
        } else if num == u64::from(&config.awake_host_shortcut) {
            self.wake_up_host();
        } else {
            return true;
        }

        self.send_telemetry(parent_id);
        false
    }

    async fn send_exit(&mut self, parent_id: Uuid) {
        let event = Event::with_source_id(self.state.id.clone(), DomainEvent::Exit, parent_id);
        self.events.push(event);
    }

    async fn toggle_mode(&mut self, parent_id: Uuid) {
        let new_mode = self.state.mode().await.toggle();
        debug!("Switching mode to {:?}", new_mode);
        self.state.set_mode(new_mode).await;
        let event = Event::with_source_id(
            self.state.id.clone(),
            DomainEvent::ModeChange(new_mode),
            parent_id,
        );
        self.events.push(event);
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

    fn send_telemetry(&mut self, parent_id: Uuid) {
        let config = self.state.config();
        if config.enable_telemetry {
            let start_time = self.state.start_time();
            let event = Event::with_source_id(
                self.state.id.clone(),
                DomainEvent::ReportConsumed(get_delta_since_start(&start_time)),
                parent_id,
            );
            self.events.push(event);
        }
    }
}

#[async_trait::async_trait]
impl Processor for SystemShortcutProcessor {
    async fn process(&mut self, input: Vec<Event>) -> Vec<Event> {
        for event in input.into_iter() {
            match &event.payload {
                DomainEvent::HidReport(report) => {
                    if self.handle_report(&report, event.id).await {
                        self.events.push(event);
                    }
                }
                _ => self.events.push(event),
            }
        }
        std::mem::take(&mut self.events)
    }
}
