use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

use crate::domain::{AppMsg, traits::UiApp};

use super::State;

pub struct Charonsay {
    state: State,
}

impl Charonsay {
    pub fn new_box() -> Box<dyn UiApp> {
        Box::new(Self {
            state: State::default(),
        })
    }
}

impl UiApp for Charonsay {
    fn id(&self) -> &'static str {
        "charonsay"
    }

    fn update(&mut self, msg: &AppMsg) {}

    fn render(&self, f: &mut Frame) {
        let body = format!("{}\n\n{}", self.state.art, self.state.wisdom);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.state.title.clone());
        let vspace = ((f.area().height as usize - body.lines().count()) / 2) - 0;
        let text = format!("{}{}", "\n".repeat(vspace), body);
        let text = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(text, f.area());
    }
}
