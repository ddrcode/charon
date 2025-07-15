use std::time::Instant;

pub fn get_delta_since_start(start: &Instant) -> u128 {
    let now = Instant::now();
    let delta = now.duration_since(*start);
    delta.as_micros()
}
