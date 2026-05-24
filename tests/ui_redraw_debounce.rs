use std::time::{Duration, Instant};

use tersse::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE;
use tersse::pure::resize_debounce::{debounce_deadline, debounce_has_elapsed};

#[test]
fn ui_redraw_debounce_queue_update_is_positive() {
    assert!(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE > 0);
}

#[test]
fn ui_redraw_debounce_queue_update_waits_for_quiet_period() {
    let start = Instant::now();
    let deadline = debounce_deadline(start, Duration::from_millis(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE));
    assert!(!debounce_has_elapsed(deadline, start));
    assert!(debounce_has_elapsed(
        deadline,
        start + Duration::from_millis(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE)
    ));
}
