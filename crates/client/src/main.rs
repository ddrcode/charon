pub mod app;
pub mod apps;
pub mod client;
pub mod config;
pub mod domain;
pub mod editor;
pub mod repository;
pub mod root;
pub mod screen;
pub mod tui;
pub mod util;

use std::{collections::HashMap, sync::Arc};

use tokio::net::UnixStream;

use crate::{
    apps::Charonsay,
    client::CharonClient,
    config::AppConfig,
    domain::{Context, traits::UiApp},
    root::AppManager,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = app::AppState::new();
    let sock = UnixStream::connect("/tmp/charon.sock").await.unwrap();
    let ctx = Arc::new(Context {
        config: AppConfig::default(),
    });

    let apps: HashMap<&'static str, Box<dyn UiApp + Send + Sync>> =
        vec![Charonsay::new_box(ctx.clone())]
            .into_iter()
            .map(|app| (app.id(), app))
            .collect();

    let app_mngr = AppManager::new(apps, "charonsay");

    let mut charon = CharonClient::new(app_mngr, state, sock);
    charon.run().await?;
    Ok(())
}
