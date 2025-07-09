use super::DomainEvent;

#[derive(Debug, PartialEq)]
pub enum Topic {
    System,
    TextInput,
    KeyInput,
    KeyOutput,
    Stats,
    Monitoring,
}

impl From<&DomainEvent> for Topic {
    fn from(value: &DomainEvent) -> Self {
        use Topic::*;
        match value {
            DomainEvent::KeyPress(_) => KeyInput,
            DomainEvent::KeyRelease(_) => KeyInput,
            DomainEvent::HidReport(_) => KeyOutput,
            DomainEvent::SendText(_) => TextInput,
            DomainEvent::SendFile(_) => TextInput,
            DomainEvent::TextSent => Monitoring,
            DomainEvent::CurrentStats(_) => Stats,
            DomainEvent::ModeChange(_) => System,
            DomainEvent::Exit => System,
        }
    }
}
