use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};
use tracing::{error, info};

use super::State;
use crate::{
    domain::{AppMsg, Command, Context, traits::UiApp},
    repository::metrics::MetricsRepository,
};

pub struct Stats {
    ctx: Arc<Context>,
    metrics: MetricsRepository,
    state: State,
}

impl Stats {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self {
            ctx,
            metrics: MetricsRepository::new(),
            state: State::default(),
        })
    }

    fn render_chart(&self, f: &mut Frame, rect: Rect) {
        let datasets = vec![
            // Scatter chart
            Dataset::default()
                .name("Avg")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().cyan())
                .data(&[
                    (0.0, 32.0),
                    (1.0, 29.0),
                    (2.0, 39.1),
                    (3.0, 30.0),
                    (4.0, 27.5),
                    (5.0, 28.0),
                    (6.0, 32.3),
                ]),
            Dataset::default()
                .name("Max")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().magenta())
                .data(&[
                    (0.0, 42.0),
                    (1.0, 53.0),
                    (2.0, 49.1),
                    (3.0, 59.0),
                    (4.0, 47.5),
                    (5.0, 48.0),
                    (6.0, 42.3),
                ]),
        ];

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            // .title("Day".red())
            .style(Style::default().white())
            .bounds([0.0, 6.0])
            .labels(["Mo", "Th", "Su"]);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title("WPM".red())
            .style(Style::default().white())
            .bounds([20.0, 60.0])
            .labels(["20", "40", "60"]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(
                Block::new()
                    .title("WPM (last week)".bold())
                    .title_alignment(Alignment::Center),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        f.render_widget(chart, rect);
    }
}

#[async_trait::async_trait]
impl UiApp for Stats {
    fn id(&self) -> &'static str {
        "stats"
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        match msg {
            AppMsg::Activate => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.state.resolution = 100;
                self.state.start = now - self.state.period.val();
                self.state.data1 = self
                    .metrics
                    .avg_wpm_for_range(self.state.start, self.state.end(), self.state.step())
                    .await
                    .ok();
                self.state.data2 = self
                    .metrics
                    .max_wpm_for_range(self.state.start, self.state.end(), self.state.step())
                    .await
                    .ok();
                info!("!!!! {:?}", self.state);
                Some(Command::Render)
            }
            AppMsg::Backend(DomainEvent::KeyRelease(key, _)) => match *key {
                KeyCode::KEY_ESC => Some(Command::RunApp("menu")),
                _ => None,
            },
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(99), Constraint::Length(1)])
            .split(f.area());

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1)].repeat(4))
            .split(layout[1]);

        self.render_chart(f, layout[0]);
        f.render_widget(" \u{f06c1} \u{f06c2} Prev/next".gray(), bottom_layout[0]);
        f.render_widget("\u{f06c3} \u{f06c0} Period".gray(), bottom_layout[1]);
        f.render_widget(
            "\u{f1050} Next stat".gray().into_right_aligned_line(),
            bottom_layout[2],
        );
        f.render_widget(
            "\u{f12b7} Exit ".gray().into_right_aligned_line(),
            bottom_layout[3],
        );
    }
}
