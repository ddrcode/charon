use std::sync::Arc;

use async_trait::async_trait;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use tokio::fs::read_to_string;
use tracing::error;

use crate::{
    apps::keymap::KeyboardLayout,
    domain::{AppEvent, Command, Context, traits::UiApp},
};

pub struct Keymap {
    ctx: Arc<Context>,
    layout: KeyboardLayout,
}

impl Keymap {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self {
            ctx,
            layout: KeyboardLayout::default(),
        })
    }

    async fn load_layout(&mut self) {
        if let Ok(layout) = read_to_string(self.ctx.config.keyboard_layout_file.clone())
            .await
            .inspect_err(|err| error!("Error loading layout file: {err}"))
        {
            self.layout = KeyboardLayout::from_str(&layout);
        }
    }
}

#[async_trait]
impl UiApp for Keymap {
    fn id(&self) -> &'static str {
        "keymap"
    }

    async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        match msg {
            AppEvent::Activate => {
                self.load_layout().await;
                None
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Percentage(99)])
            .split(f.area());

        let mut lines: Vec<Line> = Vec::new();
        let mut line = Line::default();
        for (part, is_key) in self.layout.parts() {
            if is_key {
                line.push_span(part);
            } else {
                for (i, p) in part.split('\n').enumerate() {
                    if i > 0 {
                        lines.push(std::mem::take(&mut line));
                    }
                    line.push_span(Span::styled(
                        p.to_string(),
                        Style::default().fg(Color::Gray),
                    ));
                }
            }
        }
        lines.push(line);
        let p = Paragraph::new(Text::from(lines));
        f.render_widget(p, rows[1]);

        let title = Paragraph::new("Keyboard layout".bold());
        f.render_widget(title, rows[0]);
    }
}
