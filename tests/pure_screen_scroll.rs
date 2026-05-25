use tersse::pure::screen_scroll::{
    apply_scroll_to_y, clamp_screen_scroll, screen_content_height, screen_viewport_height,
    scroll_screen_down, scroll_screen_up,
};

#[test]
fn screen_content_height_uses_tallest_element() {
    assert_eq!(screen_content_height(&[]), 0);
    assert_eq!(screen_content_height(&[(2, 1), (5, 3)]), 8);
    assert_eq!(screen_content_height(&[(10, 5)]), 15);
}

#[test]
fn screen_viewport_height_matches_usable_rows() {
    assert_eq!(screen_viewport_height(23), 23);
    assert_eq!(screen_viewport_height(1), 1);
}

#[test]
fn screen_scroll_clamps_and_moves_by_line() {
    assert_eq!(scroll_screen_up(0), 0);
    assert_eq!(scroll_screen_up(3), 2);
    assert_eq!(scroll_screen_down(0, 10, 3), 1);
    assert_eq!(scroll_screen_down(7, 10, 3), 7);
    assert_eq!(clamp_screen_scroll(99, 10, 3), 7);
}

#[test]
fn apply_scroll_to_y_allows_negative_draw_rows() {
    assert_eq!(apply_scroll_to_y(5, 3), 2);
    assert_eq!(apply_scroll_to_y(1, 5), -4);
}
