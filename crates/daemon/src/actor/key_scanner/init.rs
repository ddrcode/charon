use tokio::sync::mpsc::{Receiver, Sender};

use super::KeyScanner;
use crate::domain::Event;

pub async fn spawn_key_scanner(tx: Sender<Event>, rx: Receiver<Event>) {
    let mut scanner = KeyScanner::new(tx, rx);
    tokio::task::spawn_blocking(move || {
        scanner.run();
    });
}
