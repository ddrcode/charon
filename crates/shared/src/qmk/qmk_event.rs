use eyre::{OptionExt, eyre};
use serde::{Deserialize, Serialize};

use crate::{event::Mode, qmk::QMKRecord};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QMKEvent {
    Echo([u8; 32]),
    LayerChange(u8),
    KeyEvent(QMKRecord),
    ModeChange(Mode),
    ToggleMode,
}

impl QMKEvent {
    pub fn to_bytes(self) -> [u8; 32] {
        use QMKEvent::*;
        let mut bytes = [0u8; 32];
        match self {
            Echo(b) => return b,
            LayerChange(layer) => {
                bytes[0] = 0x02;
                bytes[1] = layer;
            }
            KeyEvent(qmkrecord) => {
                bytes[0] = 0x03;
                qmkrecord
                    .to_bytes()
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, byte)| bytes[i + 1] = byte);
            }
            ModeChange(mode) => {
                bytes[0] = 0x04;
                bytes[1] = mode as u8;
            }
            ToggleMode => {
                bytes[0] = 0x05;
            }
        }
        bytes
    }
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
