use std::borrow::Cow;

use charon_lib::event::DomainEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum Command {
    Quit,
    ExitApp,
    Render,
    RunApp(&'static str),
    SendEvent(DomainEvent),
    SuspendApp,
    ResumeApp,
    SuspendTUI,
    ResumeTUI,
    ClearScreen,
    RunExternal(Cow<'static, str>, Vec<String>),
}
