#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Mode {
    PassThrough,
    InApp,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::PassThrough
    }
}
