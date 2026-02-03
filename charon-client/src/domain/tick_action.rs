use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Default)]
pub enum TickAction {
    Quit,
    Upgrade,
    Suspend,
    #[default]
    None,
}
