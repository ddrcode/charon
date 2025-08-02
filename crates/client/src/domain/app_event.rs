use std::time::Duration;

use charon_lib::event::DomainEvent;
use crossterm::event::KeyEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum AppEvent {
    Activate,
    Tick,
    TimerTick(Duration),
    Key(KeyEvent),
    Backend(DomainEvent),
    Quit,
    Resize(u16, u16),
}
