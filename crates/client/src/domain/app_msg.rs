use std::time::Duration;

use charon_lib::event::DomainEvent;

pub enum AppMsg {
    TimerTick(Duration),
    Backend(DomainEvent),
}
