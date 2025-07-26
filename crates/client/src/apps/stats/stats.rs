use std::{borrow::Cow, sync::Arc};

use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};
use tracing::warn;

use super::{State, StatsPeriod};
use crate::{
    apps::stats::StatType,
    domain::{AppMsg, Command, Context, traits::UiApp},
    repository::metrics::{MetricsRepository, RangeResponse},
};

pub struct Stats {
    _ctx: Arc<Context>,
    metrics: MetricsRepository,
    state: State,
}

impl Stats {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        Box::new(Self {
            _ctx: ctx,
            metrics: MetricsRepository::new(25),
            state: State::default(),
        })
    }

    fn title(&self) -> Cow<'static, str> {
        use super::StatsPeriod::*;
        use chrono::prelude::*;
        let shift = self.state.shift;
        let date: DateTime<Local> = Local.timestamp_opt(self.state.start as i64, 0).unwrap();
        match self.state.period {
            Day if shift == 0 => "today".into(),
            Day if shift == 1 => "yesterday".into(),
            Day => date.format("%v").to_string().into(),
            Week if shift == 0 => "this week".into(),
            Week if shift == 1 => "last week".into(),
            Week => format!("week starting {}", date.format("%v")).into(),
            Month if shift == 0 => "this month".into(),
            Month if shift == 1 => "last month".into(),
            Month => date.format("%B, %Y").to_string().into(),
            Year if shift == 0 => "this year".into(),
            Year if shift == 1 => "last year".into(),
            Year => date.format("%Y").to_string().into(),
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
        let data1 = self.state.data1.as_deref().unwrap();
        let data2 = self.state.data2.as_deref().unwrap();
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

    fn normalize(&mut self, data: anyhow::Result<RangeResponse>) -> Option<Vec<(f64, f64)>> {
        let (start, end, _) = self.state.start_end_step();
        match data {
            Ok(data) => Some(data.normalize_with_zeros(start..end, self.state.resolution)),
            Err(err) => {
                warn!("Failed fetching metrics (data1): {err:?}");
                None
            }
        }
    }

    async fn update_data(&mut self) -> Option<Command> {
        let (start, end, step) = self.state.start_end_step();
        let (data1, data2) = match self.state.stat_type {
            StatType::Wpm => (
                self.normalize(self.metrics.avg_wpm_for_range(start, end, step).await),
                self.normalize(self.metrics.max_wpm_for_range(start, end, step).await),
            ),
            StatType::TotalKeyPress => (
                self.normalize(
                    self.metrics
                        .total_key_presses_for_range(start, end, step)
                        .await,
                ),
                self.normalize(
                    self.metrics
                        .total_key_presses_for_range(start, end, step)
                        .await,
                ),
            ),
        };
        self.state.data1 = data1;
        self.state.data2 = data2;
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
                self.state.reset_with_period(StatsPeriod::Day);
                self.update_data().await
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
                KeyCode::KEY_UP => {
                    self.state.reset_with_period(self.state.period.next());
                    self.update_data().await
                }
                KeyCode::KEY_DOWN => {
                    self.state.reset_with_period(self.state.period.prev());
                    self.update_data().await
                }
                KeyCode::KEY_SPACE => {
                    self.state.reset_with_type(self.state.stat_type.next());
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
