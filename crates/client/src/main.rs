pub mod app;
pub mod apps;
pub mod client;
pub mod domain;
pub mod editor;
pub mod repository;
pub mod root;
pub mod screen;
pub mod tui;
pub mod util;

use std::collections::HashMap;

use tokio::net::UnixStream;

use crate::{apps::Charonsay, client::CharonClient, domain::traits::UiApp, root::AppManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = app::AppState::new();
    let sock = UnixStream::connect("/tmp/charon.sock").await.unwrap();

    let apps: HashMap<&'static str, Box<dyn UiApp>> = vec![Charonsay::new_box()]
        .into_iter()
        .map(|app| (app.id(), app))
        .collect();

    let app_mngr = AppManager::new(apps, "charonsay");

    let mut charon = CharonClient::new(app_mngr, state, sock);
    charon.run().await?;
    Ok(())
}
