#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum AppPhase {
    #[default]
    Uninitialized,
    Started,
    Running,
    Closing,
    Closed,
    Sending,
    Finishing,
    Done,
}
