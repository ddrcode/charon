// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    apps::menu::menu_item::MenuItem,
    components::centered_area,
    domain::{AppEvent, Command, Context, traits::UiApp},
};

pub struct Menu {
    _ctx: Arc<Context>,
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new_box(ctx: Arc<Context>, items: Vec<MenuItem>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self { _ctx: ctx, items })
    }

    fn render_app_icon(&self, f: &mut Frame, app: &MenuItem, area: Rect) {
        let icon = Span::styled(
            app.icon.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let name = Span::styled(
            app.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let shortcut = Span::styled(format!("({})", app.shortcut), Style::default().gray());

        let lines = vec![
            Line::from(icon),
            Line::from(name),
            Line::from(Span::raw(" ")),
            Line::from(shortcut),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Cyan));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}

#[async_trait::async_trait]
impl UiApp for Menu {
    fn id(&self) -> &'static str {
        "menu"
    }

    async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        match msg {
            AppEvent::Key(key) if key.is_press() && key.modifiers == KeyModifiers::NONE => {
                let cmd = match key.code {
                    KeyCode::Char('e') => Command::RunApp("editor"),
                    KeyCode::Char('k') => Command::RunApp("keymap"),
                    KeyCode::Char('p') => Command::RunApp("password"),
                    KeyCode::Char('q') => Command::Quit,
                    KeyCode::Char('u') => Command::Upgrade,
                    KeyCode::Char('s') => Command::RunApp("stats"),
                    _ => return None,
                };
                Some(cmd)
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame) {
        let (item_width, item_hmargin) = (13u16, 2u16);
        let (item_height, item_vmargin) = (6u16, 1u16);

        let hcount: usize = (f.area().width / (item_width + item_hmargin)).into();
        let width = (hcount as u16 * (item_width + item_hmargin)) - item_hmargin;

        let vcount: usize = (f.area().height / (item_height + item_vmargin)).into();
        let height = (vcount as u16 * (item_height + item_vmargin)) - item_vmargin;

        let area = centered_area(f.area(), width, height);

        let constraints = [
            Constraint::Length(item_height),
            Constraint::Length(item_vmargin),
        ]
        .repeat(vcount)
        .into_iter()
        .take(vcount * 2 - 1);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let constraints = [
            Constraint::Length(item_width),
            Constraint::Length(item_hmargin),
        ]
        .repeat(hcount)
        .into_iter()
        .take(hcount * 2 - 1);

        let row_layouts: Vec<Vec<Rect>> = rows
            .iter()
            .step_by(2)
            .map(|row| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints.clone())
                    .split(*row)
                    .iter()
                    .step_by(2)
                    .copied()
                    .collect::<Vec<Rect>>() // not &Rect
            })
            .collect();

        for (app, area) in self.items.iter().zip(row_layouts.into_iter().flatten()) {
            self.render_app_icon(f, app, area);
        }
    }
}
