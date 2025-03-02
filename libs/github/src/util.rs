use std::time::{SystemTime, UNIX_EPOCH};

/// The current unix time in ms
pub fn unix_time_in_seconds() -> u64 {
    let current_time = SystemTime::now();
    current_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}
