use std::{borrow::Cow, convert::identity, sync::Arc};

use charon_lib::{event::DomainEvent, util::number::integer_digit_count};
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

    fn period_name(&self) -> Cow<'static, str> {
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

    fn dataset_style(dataset_id: usize) -> Style {
        match dataset_id {
            0 => Style::default().cyan(),
            1 => Style::default().yellow(),
            _ => Style::default().magenta(),
        }
    }

    fn set_dataset_name<'a>(&self, dataset: Dataset<'a>, dataset_id: usize) -> Dataset<'a> {
        let name = match (&self.state.stat_type, dataset_id) {
            (StatType::Wpm, 0) => "Avg",
            (StatType::Wpm, 1) => "Max",
            _ => return dataset,
        };
        dataset.name(name)
    }

    fn compute_y_max(&self) -> f64 {
        self.state
            .data
            .iter()
            .flatten()
            .map(|(_, val)| val)
            .copied()
            .reduce(f64::max)
            .map(|m| {
                let base = 10_u64.pow(integer_digit_count(m) - 1) as f64;
                (m / base).ceil() * base
            })
            .unwrap_or(0.0)
    }

    fn render_chart(&self, f: &mut Frame, rect: Rect) {
        let title = self.state.stat_type.to_string();
        let len = self.state.data.iter().map(|d| d.len()).max().unwrap_or(0);
        let max = self.compute_y_max();

        let datasets = self
            .state
            .data
            .iter()
            .enumerate()
            .map(|(idx, d)| {
                let ds = Dataset::default()
                    .marker(symbols::Marker::Dot)
                    .graph_type(GraphType::Line)
                    .style(Self::dataset_style(idx))
                    .data(d.as_ref());
                self.set_dataset_name(ds, idx)
            })
            .collect();

        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, len as f64])
            .labels::<Vec<&str>>(self.x_axis_labels());

        let y_axis = Axis::default()
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
                    .title(format!("{title} ({})", self.period_name()).bold())
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
        self.state.data = match self.state.stat_type {
            StatType::Wpm => vec![
                self.normalize(self.metrics.avg_wpm_for_range(start, end, step).await),
                self.normalize(self.metrics.max_wpm_for_range(start, end, step).await),
            ],
            StatType::TotalKeyPress => vec![
                self.normalize(
                    self.metrics
                        .total_key_presses_for_range(start, end, step)
                        .await,
                ),
            ],
        }
        .into_iter()
        .filter_map(identity)
        .collect();
        Some(Command::Render)
    }

    async fn update_after<F>(&mut self, op: F) -> Option<Command>
    where
        F: FnOnce(&mut State),
    {
        op(&mut self.state);
        self.update_data().await
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
                KeyCode::KEY_LEFT => self.update_after(|state| state.prev()).await,
                KeyCode::KEY_RIGHT => self.update_after(|state| state.next()).await,
                KeyCode::KEY_UP => {
                    self.update_after(|state| state.reset_with_period(state.period.next()))
                        .await
                }
                KeyCode::KEY_DOWN => {
                    self.update_after(|state| state.reset_with_period(state.period.prev()))
                        .await
                }
                KeyCode::KEY_SPACE => {
                    self.update_after(|state| state.reset_with_type(state.stat_type.next()))
                        .await
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
