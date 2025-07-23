use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;

use crate::util::time::nanos_since_start;

use super::DomainEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: u64,
    pub sender: Cow<'static, str>,
    pub payload: DomainEvent,
    pub source_event_id: Option<Uuid>,
}

impl Event {
    pub fn new(sender: Cow<'static, str>, payload: DomainEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: nanos_since_start(),
            sender: sender.into(),
            payload,
            source_event_id: None,
        }
    }

    pub fn with_source_id(
        sender: Cow<'static, str>,
        payload: DomainEvent,
        source_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: nanos_since_start(),
            sender: sender.into(),
            payload,
            source_event_id: Some(source_id),
        }
    }
}
