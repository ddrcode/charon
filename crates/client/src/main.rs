pub mod app;
pub mod client;
pub mod domain;
pub mod editor;
pub mod repository;
pub mod screen;
pub mod tui;
pub mod util;

use tokio::net::UnixStream;

use crate::client::CharonClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = app::AppState::new();
    let sock = UnixStream::connect("/tmp/charon.sock").await.unwrap();
    let mut charon = CharonClient::new(state, sock);
    charon.run().await?;
    Ok(())
}
