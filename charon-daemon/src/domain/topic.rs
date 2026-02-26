// SPDX-License-Identifier: GPL-3.0-or-later
use maiko::{Label, OverflowPolicy};

use super::CharonEvent;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Label)]
pub enum Topic {
    TextInput,
    KeyInput,
    KeyOutput,
    Stats,
    Monitoring,
    Telemetry,
    Keyboard,
    Client,
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

            ModeChange(_) => Client,
            Sleep => Client,
            WakeUp => Client,

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

    fn overflow_policy(&self) -> maiko::OverflowPolicy {
        match self {
            Topic::TextInput => OverflowPolicy::Block,
            Topic::KeyInput => OverflowPolicy::Block,
            Topic::KeyOutput => OverflowPolicy::Block,
            Topic::Stats => OverflowPolicy::Drop,
            Topic::Monitoring => OverflowPolicy::Block,
            Topic::Telemetry => OverflowPolicy::Drop,
            Topic::Keyboard => OverflowPolicy::Block,
            Topic::Client => OverflowPolicy::Drop,
        }
    }
}
