use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

pub fn draw_menu(f: &mut Frame) {
    let block = Block::default().title("Menu").borders(Borders::ALL);
    let items = vec![ListItem::new("e: Open Editor"), ListItem::new("q: Quit")];
    let list = List::new(items).block(block);
    f.render_widget(list, f.size());
}
