use std::time::SystemTime;
use uuid::Uuid;

use super::DomainEvent;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub payload: DomainEvent,
}

impl Event {
    pub fn new(payload: DomainEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            payload,
        }
    }
}
