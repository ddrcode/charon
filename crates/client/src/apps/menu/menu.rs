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
    components::centered_area,
    domain::{AppMsg, Command, Context, traits::UiApp},
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

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        match msg {
            AppMsg::Backend(DomainEvent::KeyRelease(key, _)) => match *key {
                KeyCode::KEY_E => Some(Command::RunApp("editor")),
                KeyCode::KEY_P => Some(Command::RunApp("password")),
                KeyCode::KEY_Q => Some(Command::Exit),
                KeyCode::KEY_S => Some(Command::RunApp("stats")),
                _ => None,
            },
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
        .repeat(hcount as usize)
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
                    .into_iter()
                    .step_by(2)
                    .copied()
                    .collect::<Vec<Rect>>() // not &Rect
            })
            .collect();

        for (app, area) in self.items.iter().zip(row_layouts.into_iter().flatten()) {
            self.render_app_icon(f, app, area.clone());
        }
    }
}
