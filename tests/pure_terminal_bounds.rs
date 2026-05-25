use tersse::pure::terminal_bounds::{
    clip_height_at_terminal, clip_rect, clip_str_to_cols, cols_for_printing, cols_visible_from,
    content_max_y, drawable_rows_in_span, max_element_row_cols, row_is_visible,
    rows_visible_from, text_input_draw_line_indices, visible_element_line_range,
};

#[test]
fn content_max_y_reserves_one_row() {
    assert_eq!(content_max_y(23), 22);
    assert_eq!(content_max_y(0), 0);
}

#[test]
fn clip_height_truncates_tall_element_to_terminal_bottom() {
    assert_eq!(clip_height_at_terminal(0, 100, 23), 23);
    assert_eq!(clip_height_at_terminal(10, 50, 23), 13);
    assert_eq!(clip_height_at_terminal(30, 5, 23), 0);
}

#[test]
fn clip_height_for_anchor_above_viewport() {
    assert_eq!(rows_visible_from(-3, 23), 23);
    assert_eq!(clip_height_at_terminal(-3, 10, 23), 10);
    assert_eq!(clip_height_at_terminal(-3, 30, 23), 23);
}

#[test]
fn visible_element_line_range_matches_viewport() {
    assert_eq!(visible_element_line_range(4, 100, 23), 0..19);
    assert_eq!(visible_element_line_range(-3, 100, 23), 3..26);
    assert_eq!(visible_element_line_range(30, 5, 23), 0..0);
}

#[test]
fn clip_rect_at_bottom_right() {
    assert_eq!(clip_rect(0, 20, 80, 12, 79, 23), (79, 3));
    assert_eq!(clip_rect(70, 22, 20, 5, 79, 23), (9, 1));
}

#[test]
fn clip_rect_off_screen() {
    assert_eq!(clip_rect(0, 30, 10, 1, 79, 23), (0, 0));
}

#[test]
fn clip_str_respects_cols() {
    assert_eq!(clip_str_to_cols("abcdef", 3), "abc");
}

#[test]
fn cols_for_printing_bottom_row() {
    assert_eq!(cols_for_printing(0, 79, 21, 23), 79);
    assert_eq!(cols_for_printing(0, 79, 22, 23), 79);
    assert_eq!(cols_for_printing(0, 79, 23, 23), 0);
}

#[test]
fn row_is_visible_within_terminal() {
    assert!(row_is_visible(0, 23));
    assert!(row_is_visible(22, 23));
    assert!(!row_is_visible(23, 23));
    assert!(!row_is_visible(-1, 23));
}

#[test]
fn max_element_row_cols_respects_width_and_terminal_edges() {
    assert_eq!(max_element_row_cols(0, 79, 21, 23, 80), 79);
    assert_eq!(max_element_row_cols(0, 79, 22, 23, 80), 79);
    assert_eq!(max_element_row_cols(0, 79, 22, 23, 20), 20);
    assert_eq!(max_element_row_cols(75, 79, 21, 23, 20), 4);
    assert_eq!(max_element_row_cols(75, 79, 22, 23, 20), 4);
}

#[test]
fn visible_counts_are_zero_when_anchor_is_off_screen() {
    assert_eq!(cols_visible_from(80, 79), 0);
    assert_eq!(rows_visible_from(23, 23), 0);
}

#[test]
fn clip_str_empty_when_no_columns_available() {
    assert_eq!(clip_str_to_cols("abcdef", 0), "");
}

#[test]
fn drawable_rows_in_span_skips_blocked_rows() {
    let blocked = |y: i32| (0..3).contains(&y);
    assert_eq!(drawable_rows_in_span(0, 10, 23, blocked), 7);
    assert_eq!(drawable_rows_in_span(4, 10, 23, |_| false), 10);
}

#[test]
fn text_input_draw_line_indices_intersects_scroll_with_on_screen_lines() {
    // Anchor screen y=-10 with viewport=19 means only viewport rows 10..=18 are visible.
    let on_screen = visible_element_line_range(-10, 50, 23);
    assert_eq!(on_screen, 10..33);
    let draw = text_input_draw_line_indices(-10, 50, 10, 19, 23);
    assert_eq!(draw, 20..29);
    let draw = text_input_draw_line_indices(-10, 50, 25, 19, 23);
    assert_eq!(draw, 35..44);
}

#[test]
fn text_input_draw_line_indices_keep_full_viewport_when_scrolled() {
    let draw = text_input_draw_line_indices(0, 100, 10, 19, 23);
    assert_eq!(draw, 10..29);
}
