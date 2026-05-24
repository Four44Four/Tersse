use std::time::{Duration, Instant};

use tersse::pure::resize_debounce::{debounce_deadline, debounce_has_elapsed};

#[test]
fn debounce_has_elapsed_after_window() {
    let start = Instant::now();
    let deadline = debounce_deadline(start, Duration::from_millis(500));
    assert!(!debounce_has_elapsed(deadline, start));
    assert!(debounce_has_elapsed(
        deadline,
        start + Duration::from_millis(500)
    ));
}
