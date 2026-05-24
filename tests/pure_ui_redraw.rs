use std::time::{Duration, Instant};

use tersse::pure::ui_redraw::{should_flush_debounced_queue_redraw, ElementRedrawPlan};

#[test]
fn redraw_plan_marks_only_minimum_requested_elements() {
    let mut plan = ElementRedrawPlan::default();
    plan.mark_element(4);
    plan.mark_element(4);
    plan.mark_element(8);

    assert!(plan.should_draw(4, 2));
    assert!(plan.should_draw(8, 9));
    assert!(!plan.should_draw(7, 9));
}

#[test]
fn redraw_plan_merges_redraw_from_y_to_lowest_anchor() {
    let mut plan = ElementRedrawPlan::default();
    plan.mark_from_y(9);
    plan.mark_from_y(4);
    plan.mark_element(2);

    assert!(plan.should_draw(2, 1));
    assert!(plan.should_draw(3, 4));
    assert!(plan.should_draw(5, 10));
    assert!(!plan.should_draw(6, 3));
}

#[test]
fn queue_redraw_spam_merges_into_single_flush_after_queue_empties() {
    let start = Instant::now();
    let debounce = Duration::from_millis(20);
    let deadline = Some(start + debounce);

    assert!(!should_flush_debounced_queue_redraw(
        true,
        true,
        deadline,
        start + Duration::from_millis(100),
    ));
    assert!(!should_flush_debounced_queue_redraw(
        true,
        false,
        deadline,
        start + Duration::from_millis(19),
    ));
    assert!(should_flush_debounced_queue_redraw(
        true,
        false,
        deadline,
        start + Duration::from_millis(20),
    ));
}

#[test]
fn keyboard_event_can_force_merged_redraw_before_debounce_deadline() {
    let start = Instant::now();
    let deadline = Some(start + Duration::from_millis(20));

    assert!(!should_flush_debounced_queue_redraw(
        true,
        false,
        deadline,
        start + Duration::from_millis(5),
    ));
    assert!(should_flush_debounced_queue_redraw(
        true,
        false,
        None,
        start + Duration::from_millis(5),
    ));
}
