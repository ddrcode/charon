use super::ascii_art::LOGO;

pub struct State {
    pub art: &'static str,
    pub wisdom: String,
    pub title: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            art: LOGO,
            wisdom: "Charon is rowing...\n\nPress the <[magic key]> to take control".into(),
            title: "".into(),
        }
    }
}
