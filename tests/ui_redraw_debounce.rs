use std::time::{Duration, Instant};

use tersse::UI_REDRAW_DEBOUNCE_MS;
use tersse::pure::resize_debounce::{debounce_deadline, debounce_has_elapsed};

#[test]
fn ui_redraw_debounce_ms_defaults_to_fifty() {
    assert_eq!(UI_REDRAW_DEBOUNCE_MS, 50);
}

#[test]
fn ui_redraw_debounce_waits_for_quiet_period() {
    let start = Instant::now();
    let deadline = debounce_deadline(start, Duration::from_millis(UI_REDRAW_DEBOUNCE_MS));
    assert!(!debounce_has_elapsed(deadline, start));
    assert!(debounce_has_elapsed(
        deadline,
        start + Duration::from_millis(UI_REDRAW_DEBOUNCE_MS)
    ));
}
