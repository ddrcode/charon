use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

pub fn draw_pass_through(f: &mut Frame) {
    let block = Block::default().title("Charon").borders(Borders::ALL);
    let text = Paragraph::new("Charon is rowing...\nPress `q` to quit.")
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(text, f.size());
}
