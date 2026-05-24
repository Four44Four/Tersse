use std::time::{Duration, Instant};

use tersse::pure::resize_debounce::{debounce_deadline, debounce_has_elapsed};
use tersse::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;

#[test]
fn ui_redraw_debounce_queue_update_is_positive() {
    assert!(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS > 0);
}

#[test]
fn ui_redraw_debounce_queue_update_waits_for_quiet_period() {
    let start = Instant::now();
    let deadline = debounce_deadline(
        start,
        Duration::from_millis(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS),
    );
    assert!(!debounce_has_elapsed(deadline, start));
    assert!(debounce_has_elapsed(
        deadline,
        start + Duration::from_millis(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS)
    ));
}
