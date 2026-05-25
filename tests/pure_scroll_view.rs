use tersse::pure_test::scroll_view::{
    clamp_scroll_offset, content_overflows, max_scroll_offset, scroll_line_down, scroll_line_up,
    stick_to_bottom, visible_line_range,
};

#[test]
fn max_scroll_when_overflow() {
    assert_eq!(max_scroll_offset(10, 3), 7);
}

#[test]
fn clamp_and_visible_range() {
    let offset = clamp_scroll_offset(99, 10, 3);
    assert_eq!(offset, 7);
    assert_eq!(visible_line_range(offset, 3, 10), 7..10);
}

#[test]
fn stick_to_bottom_matches_max() {
    assert_eq!(stick_to_bottom(10, 3), 7);
}

#[test]
fn overflow_detection() {
    assert!(content_overflows(5, 3));
    assert!(!content_overflows(3, 3));
}

#[test]
fn zero_height_viewport_has_no_scroll_or_visible_lines() {
    assert_eq!(max_scroll_offset(10, 0), 0);
    assert_eq!(visible_line_range(2, 0, 10), 0..0);
    assert!(!content_overflows(10, 0));
}

#[test]
fn line_scrolling_moves_by_single_row_and_clamps() {
    assert_eq!(scroll_line_up(0), 0);
    assert_eq!(scroll_line_up(3), 2);
    assert_eq!(scroll_line_down(0, 10, 3), 1);
    assert_eq!(scroll_line_down(7, 10, 3), 7);
}
