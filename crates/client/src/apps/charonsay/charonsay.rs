use std::sync::Arc;

use charon_lib::event::DomainEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    apps::charonsay::{
        WisdomCategory,
        ascii_art::{BOAT, CERBERUS, GOAT, LOGO},
    },
    domain::{AppEvent, Command, Context, traits::UiApp},
    repository::WisdomDb,
    util::string::unify_line_length,
};

use super::{State, Transition};

pub struct Charonsay {
    ctx: Arc<Context>,
    state: State,
    wisdom_db: WisdomDb,
}

impl Charonsay {
    pub fn new_box(ctx: Arc<Context>) -> Box<dyn UiApp + Send + Sync> {
        let state = State::from_config(&ctx.config);
        Box::new(Self {
            ctx,
            state,
            wisdom_db: WisdomDb::from_file("data/wisdoms.json")
                .expect("Couldn't load JSON data for WisdomDB"),
        })
    }

    fn get_art(&self, category: &WisdomCategory) -> &'static str {
        pub use WisdomCategory::*;
        match category {
            Splash => LOGO,
            Idle => GOAT,
            Speed => CERBERUS,
            Charonsay => BOAT,
        }
    }

    fn get_title(&self, view: &WisdomCategory) -> &'static str {
        pub use WisdomCategory::*;
        match view {
            Splash => "",
            Idle => "Idle",
            Speed => "Typing fast!",
            Charonsay => "Charonsay",
        }
    }

    fn decide_transition(&self, msg: &AppEvent) -> Transition {
        let state = &self.state;

        match msg {
            AppEvent::Activate => Transition::ToSplash,

            AppEvent::Key(..) => {
                if state.view == WisdomCategory::Idle {
                    return Transition::ToSplash;
                }

                Transition::Stay
            }

            AppEvent::Backend(DomainEvent::CurrentStats(stats)) => {
                if stats.wpm >= self.ctx.config.fast_typing_treshold {
                    return Transition::ToSpeed;
                } else if state.view == WisdomCategory::Speed {
                    return Transition::ToCharonsay;
                }
                Transition::Stay
            }

            AppEvent::Tick(dur) => {
                if state.time_to_idle <= *dur {
                    if state.view != WisdomCategory::Idle {
                        return Transition::ToIdle;
                    }
                }
                if state.time_to_next <= *dur {
                    if state.view == WisdomCategory::Splash {
                        return Transition::ToCharonsay;
                    }
                    return Transition::CycleCurrent;
                }

                Transition::Stay
            }

            _ => Transition::Stay,
        }
    }
}

#[async_trait::async_trait]
impl UiApp for Charonsay {
    fn id(&self) -> &'static str {
        "charonsay"
    }

    async fn update(&mut self, msg: &AppEvent) -> Option<Command> {
        let transition = self.decide_transition(msg);
        let config = &self.ctx.config;
        let state = &mut self.state;
        let mut should_render = false;

        match msg {
            AppEvent::Backend(DomainEvent::KeyPress(..)) => {
                state.time_to_idle = config.idle_time;
            }
            AppEvent::Backend(DomainEvent::CurrentStats(stats)) => {
                if state.stats != *stats {
                    state.stats = stats.clone();
                    should_render = true;
                }
            }
            _ => {}
        }

        match transition {
            Transition::ToSplash => {
                state.view = WisdomCategory::Splash;
                state.time_to_next = config.splash_duration;
                state.time_to_idle = config.idle_time;
            }
            Transition::ToCharonsay => {
                state.view = WisdomCategory::Charonsay;
                state.time_to_next = config.wisdom_duration;
            }
            Transition::ToIdle => {
                state.view = WisdomCategory::Idle;
                state.time_to_next = config.wisdom_duration;
            }
            Transition::ToSpeed => {
                state.view = WisdomCategory::Speed;
                state.time_to_next = config.wisdom_duration;
            }
            Transition::CycleCurrent => {
                state.time_to_next = config.wisdom_duration;
            }
            Transition::Stay => {
                if let AppEvent::Tick(dur) = msg {
                    state.time_to_next = state.time_to_next.saturating_sub(*dur);
                    state.time_to_idle = state.time_to_idle.saturating_sub(*dur);
                }
                return if should_render {
                    Some(Command::Render)
                } else {
                    None
                };
            }
        }

        let cat = state.view;
        self.state.art = self.get_art(&cat);
        self.state.wisdom = self.wisdom_db.get_random_wisdom(cat.into()).to_string();
        self.state.title = self.get_title(&cat);

        Some(Command::Render)
    }

    fn render(&self, f: &mut Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.state.title);

        let inner_block = block.inner(f.area());

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(99), Constraint::Length(1)])
            .split(inner_block);

        let stats_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .split(layout[1]);

        let art = unify_line_length(self.state.art);
        let body = format!("{}\n\n{}", art, self.state.wisdom);
        let vspace = (inner_block.height as usize - body.lines().count()) / 2;
        let text = format!("{}{}", "\n".repeat(vspace), body);
        let text = Paragraph::new(text).alignment(Alignment::Center);

        let header = Span::styled(" WPM: ", Style::default().add_modifier(Modifier::BOLD));
        let val = Span::from(format!(
            "{} (max: {})",
            self.state.stats.wpm, self.state.stats.max_wpm
        ));
        let wpm = Paragraph::new(Line::from(vec![header, val])).fg(Color::Gray);

        let header = Span::styled(" Mileage: ", Style::default().add_modifier(Modifier::BOLD));
        let val = Span::from(format!(
            "{} ({} today) ",
            self.state.stats.total, self.state.stats.today
        ));
        let total = Paragraph::new(Line::from(vec![header, val]))
            .alignment(Alignment::Right)
            .fg(Color::Gray);

        f.render_widget(block, f.area());
        f.render_widget(text, layout[0]);
        f.render_widget(wpm, stats_layout[0]);
        f.render_widget(total, stats_layout[1]);
    }
}
