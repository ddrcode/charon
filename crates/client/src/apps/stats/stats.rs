use std::{borrow::Cow, convert::identity, sync::Arc};

use charon_lib::event::DomainEvent;
use evdev::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
};
use tracing::{info, warn};

use super::{LineChartRenderer, State, StatsPeriod};
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
        match Local.timestamp_opt(self.state.start as i64, 0) {
            chrono::offset::LocalResult::Single(date) => match self.state.period {
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
            },
            _ => "Unknown period".into(),
        }
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
        info!("AREA----> {:?}", f.area());
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(99), Constraint::Length(1)])
            .split(f.area());

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1)].repeat(4))
            .split(layout[1]);

        match self.state.stat_type {
            StatType::Wpm | StatType::TotalKeyPress => {
                LineChartRenderer::new(&self.state, self.period_name()).render(f, layout[0]);
            }
        }

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
