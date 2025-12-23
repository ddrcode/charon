use std::time::Duration;

use charon_lib::event::CharonEvent;
use crossterm::event::KeyEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum AppEvent {
    Activate,
    Tick(Duration),
    Key(KeyEvent),
    Backend(CharonEvent),
    Quit,
    Resize(u16, u16),
    ReturnFromExternal(Option<std::process::ExitStatus>),
}
