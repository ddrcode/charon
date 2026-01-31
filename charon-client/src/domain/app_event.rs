use std::time::Duration;

use charond::domain::CharonEvent;
use crossterm::event::KeyEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum AppEvent {
    Activate,
    Tick(Duration),
    Key(KeyEvent),
    Backend(CharonEvent),
    /// UI-level request to display a specific layer (used by PassThroughController)
    ShowLayer(u8),
    Quit,
    Resize(u16, u16),
    ReturnFromExternal(Option<std::process::ExitStatus>),
}
