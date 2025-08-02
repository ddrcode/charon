use charon_lib::event::DomainEvent;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum Command {
    Exit,
    Render,
    RunApp(&'static str),
    SendEvent(DomainEvent),
    SuspendApp,
    ResumeApp,
    SuspendTUI,
    ResumeTUI,
    ClearScreen,
}
