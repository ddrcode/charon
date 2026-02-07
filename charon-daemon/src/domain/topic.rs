use std::borrow::Cow;

// SPDX-License-Identifier: GPL-3.0-or-later
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
    Keyboard,
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
            Sleep => System,
            WakeUp => System,

            ReportSent => Telemetry,

            QMKEvent(..) => Monitoring,

            KeyboardAttached(..) => Keyboard,
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

    fn name(&self) -> Cow<'static, str> {
        match self {
            Topic::System => "system".into(),
            Topic::TextInput => "text_input".into(),
            Topic::KeyInput => "key_input".into(),
            Topic::KeyOutput => "key_output".into(),
            Topic::Stats => "stats".into(),
            Topic::Monitoring => "monitoring".into(),
            Topic::Telemetry => "telemetry".into(),
            Topic::Keyboard => "keyboard".into(),
        }
    }
}
