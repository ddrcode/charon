use crossbeam_channel::{Receiver, Sender};

use super::PassThrough;
use crate::domain::{Actor, Event};

pub fn spawn_pass_through(tx: Sender<Event>, tr: Receiver<Event>) {
    let mut passthrough = PassThrough::new(tx, tr);
    std::thread::spawn(move || {
        passthrough.run();
    });
}
