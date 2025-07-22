use chrono::Local;
use std::time::{Duration, SystemTime};

pub fn get_delta_since_start(start: &std::time::Instant) -> u128 {
    let now = std::time::Instant::now();
    let delta = now.duration_since(*start);
    delta.as_micros()
}

pub fn next_midnight_instant() -> tokio::time::Instant {
    let now = Local::now();
    let tomorrow = now.date_naive().succ_opt().unwrap();
    let next_midnight = tomorrow.and_hms_opt(0, 0, 0).unwrap();

    // Time until next midnight from now
    let duration = (next_midnight - now.naive_local())
        .to_std()
        .unwrap_or(Duration::from_secs(0));

    tokio::time::Instant::now() + duration
}

pub fn is_today(st: SystemTime) -> bool {
    let now = Local::now();
    let dt = chrono::DateTime::<Local>::from(st);
    now.date_naive() == dt.date_naive()
}
