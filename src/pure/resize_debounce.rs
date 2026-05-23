use std::time::{Duration, Instant};

/// Returns the instant when a debounce window ends.
pub fn debounce_deadline(from: Instant, debounce: Duration) -> Instant {
    from + debounce
}

/// Returns true when `now` is at or past `deadline`.
pub fn debounce_has_elapsed(deadline: Instant, now: Instant) -> bool {
    now >= deadline
}
