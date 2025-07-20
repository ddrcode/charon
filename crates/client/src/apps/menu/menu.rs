use std::sync::Arc;

use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    apps::menu::menu_item::MenuItem,
    domain::{AppMsg, Command, Context, traits::UiApp},
};

pub struct Menu {
    ctx: Arc<Context>,
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new_box(ctx: Arc<Context>, items: Vec<MenuItem>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self { ctx, items })
    }
}

#[async_trait::async_trait]
impl UiApp for Menu {
    fn id(&self) -> &'static str {
        "menu"
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        match msg {
            AppMsg::Backend(DomainEvent::KeyRelease(key, _)) => match *key {
                KeyCode::KEY_Q => Some(Command::Exit),
                KeyCode::KEY_E => Some(Command::RunApp("editor")),
                _ => None,
            },
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame) {
        let area = f.area();

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

        for (i, app) in self.items.iter().enumerate() {
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
}
