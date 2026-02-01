// SPDX-License-Identifier: GPL-3.0-or-later
use crate::domain::qmk::QMKEvent;

use crate::error::CharonError;

pub trait QmkDevice: Send + 'static {
    fn read_event(&mut self) -> impl Future<Output = Result<Option<QMKEvent>, CharonError>> + Send;
}
