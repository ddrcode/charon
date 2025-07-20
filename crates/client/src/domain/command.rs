use charon_lib::event::DomainEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Exit,
    Render,
    RunApp(&'static str),
    SendEvent(DomainEvent),
}
