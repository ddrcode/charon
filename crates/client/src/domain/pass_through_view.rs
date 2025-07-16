#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum PassThroughView {
    #[default]
    Splash,
    Idle,
    Speed,
    Charonsay,
}
