pub mod actor;
pub mod broker;
pub mod domain;
pub mod error;

use actor::{
    key_scanner::{self, spawn_key_scanner},
    passthrough::{self, spawn_pass_through},
};
use broker::EventBroker;
use crossbeam_channel;
use domain::Event;

fn main() {
    let (event_tx, broker_rx) = crossbeam_channel::unbounded::<Event>();
    let mut broker = EventBroker::new(broker_rx);

    let (scan_tx, scan_rx) = crossbeam_channel::unbounded::<Event>();
    broker.add_subscriber(scan_tx, key_scanner::filter);
    spawn_key_scanner(event_tx.clone(), scan_rx);

    let (pt_tx, pt_rx) = crossbeam_channel::unbounded::<Event>();
    broker.add_subscriber(pt_tx, passthrough::filter);
    spawn_pass_through(event_tx.clone(), pt_rx);

    println!("Hello, world!");
}
