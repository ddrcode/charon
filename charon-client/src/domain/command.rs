// SPDX-License-Identifier: GPL-3.0-or-later
use std::borrow::Cow;

use charond::domain::CharonEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum Command {
    Quit,
    Restart,
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
