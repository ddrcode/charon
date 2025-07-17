use std::{sync::Arc, time::Instant};

use charon_lib::event::{DomainEvent, Event, Mode};
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;
use wake_on_lan::MagicPacket;

use crate::{
    config::CharonConfig,
    domain::{ActorState, Processor},
    util::time::get_delta_since_start,
};

pub struct SystemShortcut {
    id: &'static str,
    events: Vec<Event>,
    mode: Option<Arc<RwLock<Mode>>>,
    config: Option<CharonConfig>,
    start_time: Option<Instant>,
}

impl SystemShortcut {
    pub fn new() -> Self {
        Self {
            id: "SystemShortcutProcessor",
            events: Vec::new(),
            mode: None,
            config: None,
            start_time: None,
        }
    }

    async fn handle_report(&mut self, report: &[u8; 8], parent_id: Uuid) -> bool {
        let config = self.config.as_ref().expect("Config must be set");
        let num: u64 = u64::from_ne_bytes(*report);

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
        let event = Event::with_source_id(self.id.into(), DomainEvent::Exit, parent_id);
        self.events.push(event);
    }

    async fn toggle_mode(&mut self, parent_id: Uuid) {
        let mode = self.mode.as_ref().expect("Mode must be set");
        let new_mode = mode.read().await.toggle();
        debug!("Switching mode to {:?}", mode);
        *mode.write().await = new_mode;
        let event =
            Event::with_source_id(self.id.into(), DomainEvent::ModeChange(new_mode), parent_id);
        self.events.push(event);
    }

    fn wake_up_host(&self) {
        let config = self.config.as_ref().expect("Config must be set");
        if let Some(ref mac) = config.host_mac_address {
            let packet = MagicPacket::new(mac.as_slice().try_into().expect("Incorrect MAC format"));
            match packet.send() {
                Ok(_) => info!("Magic packet sent"),
                Err(e) => error!("Error while sendimg magic packet: {e}"),
            }
        }
    }

    fn send_telemetry(&mut self, parent_id: Uuid) {
        let config = self.config.as_ref().expect("Config must be set");
        if config.enable_telemetry {
            let start_time = self.start_time.expect("start_time must be set");
            let event = Event::with_source_id(
                self.id.into(),
                DomainEvent::ReportConsumed(get_delta_since_start(&start_time)),
                parent_id,
            );
            self.events.push(event);
        }
    }
}

#[async_trait::async_trait]
impl Processor for SystemShortcut {
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

    fn set_state(&mut self, state: &ActorState) {
        self.mode = Some(state.clone_mode());
        self.config = Some(state.config().clone());
        self.start_time = Some(*state.start_time());
    }
}
