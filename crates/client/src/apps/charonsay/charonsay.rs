use std::{sync::Arc, time::Duration};

use charon_lib::event::DomainEvent;
use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    apps::charonsay::{
        WisdomCategory,
        ascii_art::{BOAT, CERBERUS, GOAT, LOGO},
    },
    domain::{AppMsg, Command, Context, traits::UiApp},
    repository::WisdomDb,
    util::string::unify_line_length,
};

use super::State;

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
}

#[async_trait::async_trait]
impl UiApp for Charonsay {
    fn id(&self) -> &'static str {
        "charonsay"
    }

    async fn update(&mut self, msg: &AppMsg) -> Option<Command> {
        let state = &mut self.state;
        let config = &self.ctx.config;
        let mut should_render = false;
        match msg {
            AppMsg::Activate => {
                state.time_to_next = config.splash_duration;
                state.time_to_idle = config.idle_time;
            }
            AppMsg::TimerTick(dur) => {
                state.time_to_next = state.time_to_next.saturating_sub(*dur);
                state.time_to_idle = state.time_to_idle.saturating_sub(*dur);
                if state.time_to_idle == Duration::ZERO {
                    state.time_to_next = Duration::ZERO;
                }
            }
            AppMsg::Backend(e) => match e {
                DomainEvent::KeyPress(..) if state.view == WisdomCategory::Idle => {
                    state.time_to_idle = config.idle_time;
                    state.time_to_next = config.splash_duration;
                    state.view = WisdomCategory::Splash;
                    should_render = true;
                }
                DomainEvent::KeyPress(..) if state.view != WisdomCategory::Idle => {
                    state.time_to_idle = config.idle_time;
                }
                _ => {}
            },
            _ => {}
        }

        if state.time_to_next == Duration::ZERO {
            let cat = if state.time_to_idle == Duration::ZERO {
                WisdomCategory::Idle
            } else {
                WisdomCategory::Charonsay
            };
            state.time_to_next = config.wisdom_duration;
            state.view = cat;
            self.state.art = self.get_art(&cat);
            self.state.wisdom = self.wisdom_db.get_random_wisdom(cat.into()).to_string();
            self.state.title = self.get_title(&cat);
            should_render = true;
        }

        if should_render {
            Some(Command::Render)
        } else {
            None
        }
    }

    fn render(&self, f: &mut Frame) {
        let art = unify_line_length(self.state.art);
        let body = format!("{}\n\n{}", art, self.state.wisdom);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.state.title);
        let vspace = (f.area().height as usize - body.lines().count()) / 2;
        let text = format!("{}{}", "\n".repeat(vspace), body);
        let text = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(text, f.area());
    }
}
