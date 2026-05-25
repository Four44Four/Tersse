use std::time::{Duration, Instant};

use tersse::pure_test::ui_redraw::{
    layout_redraw_decision, should_flush_debounced_queue_redraw, ElementLocationSnapshot,
    ElementRedrawPlan, LayoutRedrawDecision,
};

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
fn layout_redraw_decision_redraws_only_new_element_without_reflow() {
    let before = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let after = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 3, x: 6, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let parents = vec![(1, None), (2, Some(1)), (3, Some(1))];

    let (decision, ids, _) = layout_redraw_decision(3, &before, &after, &parents);
    assert_eq!(decision, LayoutRedrawDecision::Elements);
    assert_eq!(ids, vec![3]);
}

#[test]
fn layout_redraw_decision_cascades_when_unrelated_elements_move() {
    let before = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let after = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 3, x: 0, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 4 },
    ];
    let parents = vec![(1, None), (2, None), (3, None)];

    let (decision, _, anchor_y) = layout_redraw_decision(3, &before, &after, &parents);
    assert_eq!(decision, LayoutRedrawDecision::CascadeFromY);
    assert_eq!(anchor_y, 2);
}

#[test]
fn layout_redraw_decision_allows_subtree_moves_without_cascade() {
    let before = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let after = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 4 },
        ElementLocationSnapshot { id: 2, x: 0, y: 5 },
    ];
    let parents = vec![(1, None), (2, Some(1))];

    let (decision, ids, _) = layout_redraw_decision(1, &before, &after, &parents);
    assert_eq!(decision, LayoutRedrawDecision::Elements);
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn layout_redraw_decision_skips_redraw_when_only_element_removed() {
    let before = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 3, x: 6, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let after = vec![
        ElementLocationSnapshot { id: 1, x: 0, y: 2 },
        ElementLocationSnapshot { id: 2, x: 0, y: 3 },
    ];
    let parents = vec![(1, None), (2, Some(1)), (3, Some(1))];

    let (decision, ids, _) = layout_redraw_decision(3, &before, &after, &parents);
    assert_eq!(decision, LayoutRedrawDecision::Elements);
    assert!(ids.is_empty());
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
