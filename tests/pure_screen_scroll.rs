use tersse::pure::screen_scroll::{
    apply_scroll_to_y, clamp_screen_scroll, screen_content_height, screen_scroll_to_show_cursor_row,
    screen_scroll_to_show_row, screen_viewport_height, scroll_screen_down, scroll_screen_up,
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

#[test]
fn screen_scroll_to_show_row_keeps_focus_inside_viewport() {
    assert_eq!(screen_scroll_to_show_row(0, 30, 24), 0);
    assert_eq!(screen_scroll_to_show_row(23, 30, 24), 0);
    assert_eq!(screen_scroll_to_show_row(24, 30, 24), 1);
    assert_eq!(screen_scroll_to_show_row(29, 30, 24), 6);
}

#[test]
fn screen_scroll_to_show_row_keeps_element_bottom_on_screen() {
    // Input at y=4 with 25 lines ends at row 28; viewport 24 -> scroll 5.
    assert_eq!(screen_scroll_to_show_row(28, 30, 24), 5);
}

#[test]
fn screen_scroll_to_show_cursor_row_leaves_scroll_when_visible() {
    assert_eq!(screen_scroll_to_show_cursor_row(10, 5, 30, 24, 0), 5);
    assert_eq!(screen_scroll_to_show_cursor_row(23, 0, 30, 24, 0), 0);
}

#[test]
fn screen_scroll_to_show_cursor_row_pins_below_viewport_to_bottom() {
    assert_eq!(screen_scroll_to_show_cursor_row(24, 0, 30, 24, 0), 1);
    assert_eq!(screen_scroll_to_show_cursor_row(29, 0, 30, 24, 0), 6);
}

#[test]
fn screen_scroll_to_show_cursor_row_pins_above_viewport_to_top() {
    assert_eq!(screen_scroll_to_show_cursor_row(5, 10, 30, 24, 0), 5);
}
