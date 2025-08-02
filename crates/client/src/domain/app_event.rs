use std::time::Duration;

use charon_lib::event::DomainEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum AppEvent {
    Activate,
    Deactivate,
    Tick,
    TimerTick(Duration),
    Backend(DomainEvent),
    Quit,
    Resize(u16, u16),
}
