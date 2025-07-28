use std::borrow::Cow;
use std::collections::HashMap;

use ratatui::{
    prelude::*,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
};

pub struct KeyHeatmapRenderer<'a> {
    pub key_counts: &'a HashMap<String, f64>,
    pub title: Cow<'static, str>,
}

impl<'a> KeyHeatmapRenderer<'a> {
    pub fn new(key_counts: &'a HashMap<String, f64>, title: Cow<'static, str>) -> Self {
        Self { key_counts, title }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Heatmap body
            ])
            .split(area);

        let title = Paragraph::new(self.title.clone()).centered().bold();
        f.render_widget(title, layout[0]);

        let body = self.render_heatmap();
        f.render_widget(body, layout[1]);
    }

    fn name_to_label(name: &'static str) -> &'static str {
        match name {
            "~" => "GRAVE",
            "-" => "MINUS",
            "=" => "EQUAL",
            "BS" => "BACKSPACE",
            "[" => "LEFTBRACE",
            "]" => "RIGHTBRACE",
            "\\" => "BACKSLASH",
            "CAPS" => "CAPSLOCK",
            ";" => "SEMICOLON",
            "'" => "APOSTROPHE",
            "," => "COMMA",
            "." => "DOT",
            "/" => "SLASH",
            "LCTRL" => "LEFTCTRL",
            "LSHIFT" => "LEFTSHIFT",
            "LMETA" => "LEFTMETA",
            "LALT" => "LEFTALT",
            "RCTRL" => "RIGHTCTRL",
            "RSHIFT" => "RIGHTSHIFT",
            "RMETA" => "RIGHTMETA",
            "RALT" => "RIGHTALT",
            "INS" => "INSERT",
            "DEL" => "DELETE",
            "PUP" => "PAGEUP",
            "PDOWN" => "PAGEDOWN",
            _ => name,
        }
    }

    fn render_heatmap(&self) -> Paragraph {
        let rows = vec![
            vec![],
            vec![
                "ESC", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
            ],
            vec![
                "~", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "BS",
            ],
            vec![
                "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]", "\\",
            ],
            vec![
                "CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "ENTER",
            ],
            vec![
                "LSHIFT", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "RSHIFT",
            ],
            vec!["LCTRL", "LMETA", "LALT", "SPACE", "RALT", "RMETA", "RCTRL"],
            vec![],
            vec!["LEFT", "RIGHT", "UP", "DOWN"],
            vec![],
            vec!["INS", "DEL", "HOME", "PUP", "PDOWN"],
        ];

        let max = self
            .key_counts
            .values()
            .copied()
            .reduce(f64::max)
            .unwrap_or(0.0);
        let color_scale = Self::build_gradient();

        let mut lines = Vec::with_capacity(rows.len());
        for row in rows {
            let mut spans = Vec::new();
            for key in row {
                let stat_label = Self::name_to_label(key);
                let count = self.key_counts.get(stat_label).copied().unwrap_or(0.0);
                let log_count = (count + 1.0).ln();
                let log_max = (max + 1.0).ln();
                let ratio = (log_count / log_max).clamp(0.0, 1.0);
                let color_idx = (ratio * (color_scale.len() - 1) as f64).round() as usize;
                let color = color_scale[color_idx];
                let label = format!("[{key:^3}]");
                let span = Span::styled(label, Style::default().bg(color));
                spans.push(span);
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
    }

    fn build_gradient() -> Vec<Color> {
        vec![
            Color::Rgb(30, 30, 30),   // 0: almost black
            Color::Rgb(40, 50, 70),   // 1: deep navy
            Color::Rgb(50, 70, 100),  // 2: dark blue
            Color::Rgb(60, 90, 130),  // 3: mid blue
            Color::Rgb(70, 110, 150), // 4: steel blue
            Color::Rgb(60, 130, 130), // 5: teal
            Color::Rgb(50, 150, 100), // 6: sea green
            Color::Rgb(70, 170, 70),  // 7: green
            Color::Rgb(100, 190, 50), // 8: lime green
            Color::Rgb(140, 200, 50), // 9: chartreuse
            Color::Rgb(170, 190, 60), // 10: olive
            Color::Rgb(190, 170, 70), // 11: dull gold
            Color::Rgb(200, 140, 80), // 12: warm khaki
            Color::Rgb(210, 110, 90), // 13: coral
            Color::Rgb(220, 80, 100), // 14: rose red
            Color::Rgb(230, 60, 80),  // 15: blood red
            Color::Rgb(240, 40, 60),  // 16: dark red
            Color::Rgb(250, 30, 40),  // 17: hot red
            Color::Rgb(255, 0, 0),    // 18: intense red
            Color::Rgb(255, 40, 40),  // 19: overflow (very hot)
        ]
    }
}
