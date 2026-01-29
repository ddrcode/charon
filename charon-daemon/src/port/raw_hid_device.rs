use crate::error::CharonError;

#[async_trait::async_trait]
pub trait RawHidDevice {
    async fn read_packet(&mut self) -> Result<(usize, [u8; 32]), CharonError>;
}
