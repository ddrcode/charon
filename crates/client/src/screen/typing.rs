use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

const SPLASH: &'static str = "


 _______  __   __  _______  ______    _______  __    _
|       ||  | |  ||   _   ||    _ |  |       ||  |  | |
|       ||  |_|  ||  |_|  ||   | ||  |   _   ||   |_| |
|       ||       ||       ||   |_||_ |  | |  ||       |
|      _||       ||       ||    __  ||  |_|  ||  _    |
|     |_ |   _   ||   _   ||   |  | ||       || | |   |
|_______||__| |__||__| |__||___|  |_||_______||_|  |__|




Charon is rowing....

Press the <[magic key]> to take control
";

pub fn draw_pass_through(f: &mut Frame) {
    let block = Block::default().borders(Borders::ALL);
    let text = Paragraph::new(SPLASH)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(text, f.size());
}
