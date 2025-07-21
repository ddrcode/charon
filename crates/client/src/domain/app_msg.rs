use std::time::Duration;

use charon_lib::event::DomainEvent;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AppMsg {
    Activate,
    Deactivate,
    TimerTick(Duration),
    Backend(DomainEvent),
}
