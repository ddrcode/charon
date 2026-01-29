use super::CharonEvent;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Topic {
    System,
    TextInput,
    KeyInput,
    KeyOutput,
    Stats,
    Monitoring,
    Telemetry,
}

impl From<&CharonEvent> for Topic {
    fn from(value: &CharonEvent) -> Self {
        use CharonEvent::*;
        use Topic::*;
        match value {
            KeyPress(..) => KeyInput,
            KeyRelease(..) => KeyInput,
            HidReport(_) => KeyOutput,
            SendText(_) => TextInput,
            SendFile(..) => TextInput,
            TextSent => Monitoring,
            CurrentStats(_) => Stats,

            ModeChange(_) => System,
            Exit => System,
            Sleep => System,
            WakeUp => System,

            ReportSent => Telemetry,

            QMKEvent(..) => Monitoring,
        }
    }
}

impl maiko::Topic<CharonEvent> for Topic {
    fn from_event(event: &CharonEvent) -> Self
    where
        Self: Sized,
    {
        Self::from(event)
    }
}
