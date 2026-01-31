use evdev::InputEvent;

pub trait EventDevice: Send + 'static {
    fn next_event(&mut self) -> impl Future<Output = Option<InputEvent>> + Send;
    fn is_grabbed(&self) -> bool;
    fn grab(&mut self) -> std::io::Result<()>;
    fn ungrab(&mut self) -> std::io::Result<()>;
}
