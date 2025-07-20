use charon_lib::event::DomainEvent;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Command {
    Exit,
    Render,
    RunApp(&'static str),
    SendEvent(DomainEvent),
    SuspendTUI,
    ResumeTUI,
}
