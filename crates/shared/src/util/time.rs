use chrono::{Datelike, Local, TimeZone};
use std::{
    sync::LazyLock,
    time::{Duration, SystemTime},
};

static START_TIME: LazyLock<std::time::Instant> = LazyLock::new(|| std::time::Instant::now());

pub fn next_midnight_instant() -> tokio::time::Instant {
    let now = Local::now();
    let tomorrow = now.date_naive().succ_opt().unwrap();
    let next_midnight = tomorrow.and_hms_opt(0, 0, 0).unwrap();

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

pub fn nanos_since_start() -> u64 {
    let now = std::time::Instant::now();
    let delta = now.duration_since(*START_TIME);
    delta.as_nanos() as u64
}

pub fn beginning_of_today_as_unix_timestamp() -> u64 {
    let now = Local::now();
    let today_midnight = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    today_midnight.timestamp() as u64
}

pub fn beginning_of_week_as_unix_timestamp() -> u64 {
    let now = Local::now();
    let today_midnight = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let weekday = today_midnight.weekday();
    let days_since_monday = weekday.num_days_from_monday() as i64;
    let week_start = today_midnight - chrono::Duration::days(days_since_monday);
    week_start.timestamp() as u64
}
