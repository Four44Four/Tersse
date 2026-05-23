use tersse::pure::terminal_bounds::{
    clip_height_at_terminal, clip_rect, clip_str_to_cols, cols_for_printing, cols_visible_from,
    content_max_y, max_element_row_cols, row_is_visible, rows_visible_from,
    visible_element_line_range,
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
    assert_eq!(clip_rect(0, 20, 80, 12, 79, 23), (80, 3));
    assert_eq!(clip_rect(70, 22, 20, 5, 79, 23), (10, 1));
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
    assert_eq!(cols_for_printing(0, 79, 21, 23), 80);
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
    assert_eq!(max_element_row_cols(0, 79, 21, 23, 80), 80);
    assert_eq!(max_element_row_cols(0, 79, 22, 23, 80), 79);
    assert_eq!(max_element_row_cols(0, 79, 22, 23, 20), 20);
    assert_eq!(max_element_row_cols(75, 79, 21, 23, 20), 5);
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
