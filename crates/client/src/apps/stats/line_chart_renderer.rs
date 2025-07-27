use std::borrow::Cow;

use charon_lib::util::number::integer_digit_count;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};

use super::{StatType, State};

pub struct LineChartRenderer<'a> {
    state: &'a State,
    period_name: Cow<'static, str>,
}

impl<'a> LineChartRenderer<'a> {
    pub fn new(state: &'a State, period_name: Cow<'static, str>) -> Self {
        Self { state, period_name }
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

    fn set_dataset_name<'b>(&self, dataset: Dataset<'b>, dataset_id: usize) -> Dataset<'b> {
        let name = match (&self.state.stat_type, dataset_id) {
            (StatType::Wpm, 0) => "Avg",
            (StatType::Wpm, 1) => "Max",
            _ => return dataset,
        };
        dataset.name(name)
    }

    pub fn render(&self, f: &mut Frame, rect: Rect) {
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
                    .title(format!("{title} ({})", self.period_name).bold())
                    .title_alignment(Alignment::Center),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        f.render_widget(chart, rect);
    }
}
