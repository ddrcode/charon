use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use charon_lib::{event::DomainEvent, util::time::beginning_of_today_as_unix_timestamp};
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
            metrics: MetricsRepository::new(25),
            state: State::default(),
        })
    }

    fn title(&self) -> &str {
        use super::StatsPeriod::*;
        let shift = self.state.shift;
        match self.state.period {
            Day if shift == 0 => "today",
            Day if shift == 1 => "yesterday",
            Day => "FIXME",
            Week if shift == 0 => "this week",
            Week if shift == 1 => "last week",
            Week => todo!(),
            Month if shift == 0 => "this month",
            Month if shift == 1 => "last month",
            Month => todo!(),
            Year if shift == 0 => "this year",
            Year if shift == 1 => "last year",
            Year => todo!(),
        }
    }

    fn x_axis_labels(&self) -> Vec<&str> {
        use super::StatsPeriod::*;
        match self.state.period {
            Day => vec!["0", "12", "24"],
            Week => vec!["Mon", "Thu", "Sun"],
            Month => vec!["1", "15", "30"],
            Year => vec!["Jan", "Jul", "Dec"],
        }
    }

    fn render_chart(&self, f: &mut Frame, rect: Rect) {
        let data1 = self.state.data1.clone().unwrap();
        let data2 = self.state.data2.clone().unwrap();
        let len = data1.len().max(data2.len());
        let max = data2
            .iter()
            .map(|(_, val)| val)
            .copied()
            .reduce(f64::max)
            .map(|m| (m / 10.0).ceil() * 10.0)
            .unwrap_or(0.0);
        let datasets = vec![
            Dataset::default()
                .name("Avg")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().cyan())
                .data(data1.as_ref()),
            Dataset::default()
                .name("Max")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().yellow())
                .data(&data2),
        ];

        let x_axis = Axis::default()
            // .title("Day".red())
            .style(Style::default().white())
            .bounds([0.0, len as f64])
            .labels::<Vec<&str>>(self.x_axis_labels());

        let y_axis = Axis::default()
            .title("WPM".red())
            .style(Style::default().white())
            .bounds([0.0, max])
            .labels([
                "0".to_string(),
                format!("{:.0}", max / 2.0),
                format!("{max:.0}"),
            ]);

        let chart = Chart::new(datasets)
            .block(
                Block::new()
                    .title(format!("WPM ({})", self.title()).bold())
                    .title_alignment(Alignment::Center),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        f.render_widget(chart, rect);
    }

    async fn update_data(&mut self) -> Option<Command> {
        self.state.data1 = self
            .metrics
            .avg_wpm_for_range_normalized(self.state.start, self.state.end(), self.state.step())
            .await
            .ok();
        self.state.data2 = self
            .metrics
            .max_wpm_for_range_normalized(self.state.start, self.state.end(), self.state.step())
            .await
            .ok();
        Some(Command::Render)
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
                self.state.resolution = 25;
                self.state.start = beginning_of_today_as_unix_timestamp();
                self.state.data1 = self
                    .metrics
                    .avg_wpm_for_range_normalized(
                        self.state.start,
                        self.state.end(),
                        self.state.step(),
                    )
                    .await
                    .ok();
                self.state.data2 = self
                    .metrics
                    .max_wpm_for_range_normalized(
                        self.state.start,
                        self.state.end(),
                        self.state.step(),
                    )
                    .await
                    .ok();
                Some(Command::Render)
            }
            AppMsg::Backend(DomainEvent::KeyRelease(key, _)) => match *key {
                KeyCode::KEY_ESC => Some(Command::RunApp("menu")),
                KeyCode::KEY_LEFT => {
                    self.state.prev();
                    self.update_data().await
                }
                KeyCode::KEY_RIGHT => {
                    self.state.next();
                    self.update_data().await
                }
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
