use eyre::{OptionExt, eyre};
use serde::{Deserialize, Serialize};

use crate::event::{Mode, QMKRecord};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QMKEvent {
    Echo([u8; 32]),
    LayerChange(u8),
    KeyEvent(QMKRecord),
    ModeChange(Mode),
    ToggleMode,
}

impl TryFrom<[u8; 32]> for QMKEvent {
    type Error = eyre::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let to_u16 = |start| u16::from_le_bytes([bytes[start], bytes[start + 1]]);
        let event = match bytes[0] {
            0x01 => QMKEvent::Echo(bytes),
            0x02 => QMKEvent::LayerChange(bytes[1]),
            0x03 => {
                QMKEvent::KeyEvent(QMKRecord::new(to_u16(1), bytes[3] == 1, bytes[4], bytes[5]))
            }
            0x04 => QMKEvent::ModeChange(
                Mode::from_repr(bytes[1])
                    .ok_or_eyre(eyre!("Unrecognized Mode value: {}", bytes[1]))?,
            ),
            0x05 => QMKEvent::ToggleMode,
            n => return Err(eyre!("Unrecognized message id: {n}")),
        };
        Ok(event)
    }
}
