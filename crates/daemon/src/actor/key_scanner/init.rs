use std::thread;

use super::KeyScanner;
use crate::domain::{Actor, Event};
use crossbeam_channel::{Receiver, Sender};

pub fn spawn_key_scanner(tx: Sender<Event>, rx: Receiver<Event>) {
    let mut scanner = KeyScanner::new(tx, rx);
    thread::spawn(move || {
        scanner.run();
    });
}
