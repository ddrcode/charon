// use crate::error::CharonError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QMKEvent {
    LayerChange(u8),
}

// impl TryFrom<[u8; 32]> for QMKEvent {
//     type Error = CharonError;
//
//     fn try_from(value: [u8; 32]) -> Result<Self, Self::Error> {
//         match value[0] {
//             1 => Ok(QMKEvent::LayerChange(value[1])),
//             n => Err(CharonError::QMKError(format!(
//                 "Unrecognized message id: {n}"
//             ))),
//         }
//     }
// }
