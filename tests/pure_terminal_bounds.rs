use tersse::pure::terminal_bounds::{
    clip_height_at_terminal, clip_rect, clip_str_to_cols, cols_for_printing, cols_visible_from,
    rows_visible_from,
};

#[test]
fn clip_height_truncates_tall_element_to_terminal_bottom() {
    assert_eq!(clip_height_at_terminal(0, 100, 23), 24);
    assert_eq!(clip_height_at_terminal(10, 50, 23), 14);
    assert_eq!(clip_height_at_terminal(30, 5, 23), 0);
}

#[test]
fn clip_rect_at_bottom_right() {
    assert_eq!(clip_rect(0, 20, 80, 12, 79, 23), (80, 4));
    assert_eq!(clip_rect(70, 23, 20, 5, 79, 23), (10, 1));
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
    assert_eq!(cols_for_printing(0, 79, 22, 23), 80);
    assert_eq!(cols_for_printing(0, 79, 23, 23), 79);
}

#[test]
fn visible_counts_are_zero_when_anchor_is_off_screen() {
    assert_eq!(cols_visible_from(80, 79), 0);
    assert_eq!(rows_visible_from(24, 23), 0);
}

#[test]
fn clip_str_empty_when_no_columns_available() {
    assert_eq!(clip_str_to_cols("abcdef", 0), "");
}
