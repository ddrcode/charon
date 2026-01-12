use charon_lib::qmk::QMKEvent;

use crate::error::CharonError;

#[async_trait::async_trait]
pub trait QmkDevice: Send {
    async fn read_event(&mut self) -> Result<Option<QMKEvent>, CharonError>;
}
