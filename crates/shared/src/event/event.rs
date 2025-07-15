use serde::{Deserialize, Serialize};
use std::{borrow::Cow, time::SystemTime};
use uuid::Uuid;

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
    pub fn new(sender: &'static str, payload: DomainEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Self::into_millis(&SystemTime::now()),
            sender: sender.into(),
            payload,
            source_event_id: None,
        }
    }

    pub fn with_time(sender: &'static str, payload: DomainEvent, timestamp: SystemTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Self::into_millis(&timestamp),
            sender: sender.into(),
            payload,
            source_event_id: None,
        }
    }

    pub fn with_source_id(sender: &'static str, payload: DomainEvent, source_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Self::into_millis(&SystemTime::now()),
            sender: sender.into(),
            payload,
            source_event_id: Some(source_id),
        }
    }

    fn into_millis(time: &SystemTime) -> u64 {
        time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
