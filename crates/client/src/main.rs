pub mod app;
pub mod apps;
pub mod components;
pub mod config;
pub mod domain;
pub mod repository;
pub mod root;
pub mod tui;
pub mod util;

use std::{collections::HashMap, sync::Arc};

use tracing::error;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;

use crate::{
    app::App,
    apps::{
        Charonsay, Editor, Password, Stats,
        menu::{Menu, MenuItem},
    },
    config::AppConfig,
    domain::{Context, traits::UiApp},
    root::AppManager,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_logging();

    let ctx = &Arc::new(Context {
        config: AppConfig::default(),
    });

    let apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>> = vec![
        Charonsay::new_box(ctx.clone()),
        Menu::new_box(ctx.clone(), menu_items()),
        Editor::new_box(ctx.clone()),
        Stats::new_box(ctx.clone()),
        Password::new_box(ctx.clone()),
    ]
    .into_iter()
    .map(|app| (app.id(), app))
    .collect();

    std::panic::set_hook(Box::new(move |_panic_info| {
        if let Ok(mut t) = crate::tui::Tui::new() {
            if let Err(r) = t.exit() {
                error!("Unable to exit Terminal: {:?}", r);
            }
        }
    }));

    let app_mngr = AppManager::new(apps, "menu");
    let mut app = App::new(app_mngr, ctx.clone())?;
    app.run().await?;

    Ok(())
}

fn menu_items() -> Vec<MenuItem> {
    vec![
        ("Editor", '\u{ed39}', "e"),
        ("Stats", '\u{f04c5}', "s"),
        ("Passwords", '\u{f07f5}', "p"),
        ("Calendar", '\u{f07f5}', "l"),
        ("Calculator", '\u{f07f5}', "c"),
        ("Todo", '\u{f07f5}', "t"),
        ("Game", '\u{f07f5}', "g"),
        ("Quit", '\u{f0a48}', "q"),
    ]
    .iter()
    .map(MenuItem::from)
    .collect()
}

fn init_logging() {
    let file_appender = rolling::daily("logs", "charon-client.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive to flush on exit
    Box::leak(Box::new(_guard)); // or store in global if needed

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("charon=debug".parse().unwrap()),
        )
        .with_ansi(false)
        .init();
}
