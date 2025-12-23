use std::borrow::Cow;

use charon_lib::event::CharonEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum Command {
    Quit,
    ExitApp,
    Render,
    RunApp(&'static str),
    SendEvent(CharonEvent),
    SuspendApp,
    ResumeApp,
    SuspendTUI,
    ResumeTUI,
    ClearScreen,
    RunExternal(Cow<'static, str>, Vec<String>),
}
