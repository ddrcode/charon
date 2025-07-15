use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::AppState;

pub fn draw_menu(f: &mut Frame, state: &AppState) {
    let area = f.area();

    // 👉 Example: 2 rows × 2 columns grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(6),
        ])
        .split(area);

    let mut boxes = vec![];

    for row in rows.iter().step_by(2) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(13),
                Constraint::Length(2),
                Constraint::Length(13),
            ])
            .split(*row);

        for col in cols.iter().step_by(2) {
            boxes.push(col.clone());
        }
    }

    for (i, app) in state.apps.iter().enumerate() {
        if i >= boxes.len() {
            break;
        }

        let icon = Span::styled(
            app.icon.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let name = Span::styled(
            app.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let blank = Span::raw(" ");
        let shortcut = Span::styled(format!("({})", app.shortcut), Style::default().gray());

        let lines = vec![
            Line::from(icon),
            Line::from(name),
            Line::from(blank),
            Line::from(shortcut),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Cyan));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, boxes[i]);
    }
}
