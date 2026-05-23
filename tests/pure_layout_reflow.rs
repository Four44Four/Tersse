use tersse::pure::layout_reflow::{height_delta, min_y_after_change, shifted_y};

#[test]
fn height_delta_reports_growth_and_shrink() {
    assert_eq!(height_delta(2, 5), 3);
    assert_eq!(height_delta(5, 2), -3);
}

#[test]
fn min_y_after_change_uses_anchor_for_zero_height() {
    assert_eq!(min_y_after_change(7, 0), 7);
    assert_eq!(min_y_after_change(7, 3), 10);
}

#[test]
fn shifted_y_only_affects_elements_at_or_below_threshold() {
    assert_eq!(shifted_y(4, 5, 3), 4);
    assert_eq!(shifted_y(5, 5, 3), 8);
}

#[test]
fn shifted_y_saturates_at_zero_on_negative_delta() {
    assert_eq!(shifted_y(2, 2, -10), 0);
}
