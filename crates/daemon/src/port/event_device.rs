use evdev::InputEvent;

#[async_trait::async_trait]
pub trait EventDevice: Send {
    async fn next_event(&mut self) -> Option<InputEvent>;
    fn is_grabbed(&self) -> bool;
    fn grab(&mut self) -> std::io::Result<()>;
    fn ungrab(&mut self) -> std::io::Result<()>;
}
