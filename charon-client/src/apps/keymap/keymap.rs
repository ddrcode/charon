// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use tokio::fs::read_to_string;
use tracing::error;

use super::{KeyboardLayout, keycode_label::keycode_label, qmk_keymap::QmkKeymap};
use crate::domain::{AppEvent, Command, Context, traits::UiApp};

pub struct Keymap {
    ctx: Arc<Context>,
    layout: KeyboardLayout,
    qmk_keymap: Option<QmkKeymap>,
    current_layer: usize,
}

impl Keymap {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self {
            ctx,
            layout: KeyboardLayout::default(),
            qmk_keymap: None,
            current_layer: 0,
        })
    }

    async fn load_layout(&mut self) {
        // Load physical layout
        if let Ok(layout) = read_to_string(self.ctx.config.keyboard_layout_file.clone())
            .await
            .inspect_err(|err| error!("Error loading layout file: {err}"))
        {
            self.layout = KeyboardLayout::from_str(&layout);
        }

        // Load QMK keymap
        if let Ok(keymap) = QmkKeymap::load(&self.ctx.config.keymap_path()).await {
            self.qmk_keymap = Some(keymap);
            self.current_layer = 0;
            self.apply_layer_labels();
        } else {
            error!("Error loading keymap file");
        }
    }

    fn apply_layer_labels(&mut self) {
        let Some(ref keymap) = self.qmk_keymap else {
            return;
        };
        let Some(layer) = keymap.layer(self.current_layer) else {
            return;
        };

        for (i, keycode) in layer.iter().enumerate() {
            if i >= self.layout.len() {
                break;
            }
            let max_len = self.layout.key(i).len;
            let label = keycode_label(keycode, max_len);
            self.layout.set_label(i, &label);
        }
    }

    fn set_layer(&mut self, layer: usize) {
        if let Some(ref keymap) = self.qmk_keymap {
            if layer < keymap.layer_count() && layer != self.current_layer {
                self.current_layer = layer;
                self.apply_layer_labels();
            }
        }
    }

    fn next_layer(&mut self) {
        if let Some(ref keymap) = self.qmk_keymap {
            if self.current_layer + 1 < keymap.layer_count() {
                self.current_layer += 1;
                self.apply_layer_labels();
            }
        }
    }

    fn prev_layer(&mut self) {
        if self.current_layer > 0 {
            self.current_layer -= 1;
            self.apply_layer_labels();
        }
    }

    fn layer_title(&self) -> String {
        if let Some(ref keymap) = self.qmk_keymap {
            format!(
                "{} [{}]: Layer {}/{}",
                &keymap.keyboard,
                &keymap.keymap,
                self.current_layer + 1,
                keymap.layer_count(),
            )
        } else {
            "Keyboard Layout: No keymap loaded".to_string()
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Command> {
        match key.code {
            KeyCode::Esc => Some(Command::ExitApp),
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev_layer();
                Some(Command::Render)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_layer();
                Some(Command::Render)
            }
            _ => None,
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
            AppEvent::Key(key) => self.handle_key(*key),
            AppEvent::ShowLayer(layer) => {
                self.set_layer(*layer as usize);
                Some(Command::Render)
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(f.area());

        let footer = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(rows[2]);

        // Keyboard layout
        let mut lines: Vec<Line> = Vec::new();
        let mut line = Line::default();
        for (part, is_key) in self.layout.parts() {
            if is_key {
                line.push_span(Span::styled(part, Style::default().fg(Color::White)));
            } else {
                for (i, p) in part.split('\n').enumerate() {
                    if i > 0 {
                        lines.push(std::mem::take(&mut line));
                    }
                    line.push_span(Span::styled(
                        p.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
        }
        lines.push(line);
        let p = Paragraph::new(Text::from(lines));
        f.render_widget(p, rows[1]);

        // Title
        let title = Paragraph::new(self.layer_title().bold());
        f.render_widget(title, rows[0]);

        // Footer navigation
        f.render_widget(" ←/→ Layers".gray(), footer[0]);
        f.render_widget("ESC Exit".gray().into_right_aligned_line(), footer[1]);
    }
}
