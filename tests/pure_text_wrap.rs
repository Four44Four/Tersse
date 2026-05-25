use std::collections::BTreeSet;
use tersse::pure::text_wrap::{
    cursor_display_position, display_row_count, selection_highlight_cells, wrapped_line_count,
    wrapped_lines, wrapped_lines_for_display,
};

#[test]
fn wraps_at_width() {
    let lines = wrapped_lines("abcdef", 3);
    assert_eq!(lines, vec!["abc", "def"]);
}

#[test]
fn cursor_after_wrap() {
    assert_eq!(cursor_display_position("abcdef", 4, 3), (1, 1));
}

#[test]
fn empty_display_rows() {
    assert_eq!(display_row_count("", 48), 1);
    assert_eq!(display_row_count("hello", 48), 1);
}

#[test]
fn wrapped_lines_for_display_includes_blank_row_when_empty() {
    assert_eq!(wrapped_lines("", 10).len(), 0);
    assert_eq!(wrapped_lines_for_display("", 10), vec![String::new()]);
    assert_eq!(
        wrapped_lines_for_display("", 10).len(),
        display_row_count("", 10)
    );
}

#[test]
fn display_rows_after_explicit_newline() {
    assert_eq!(display_row_count("hello\n", 48), 2);
    assert_eq!(display_row_count("a\nb", 48), 2);
    assert_eq!(
        display_row_count("hello\n", 48),
        wrapped_lines_for_display("hello\n", 48).len()
    );
}

#[test]
fn scrolled_viewport_row_count_does_not_exceed_remaining_content() {
    use tersse::pure::scroll_view::{clamp_scroll_offset, visible_line_range};
    let total_lines = 30;
    let terminal_visible = 24;
    let offset = clamp_scroll_offset(11, total_lines, terminal_visible);
    assert_eq!(offset, 6);
    let range = visible_line_range(offset, terminal_visible, total_lines);
    assert_eq!(range.len(), total_lines - offset);
    assert_eq!(range.len(), 24);
    assert!(range.len() <= terminal_visible);
}

#[test]
fn display_row_count_matches_wrapped_lines_for_display() {
    for text in ["", "hello", "hello\n", "a\nb", "abcde", "hello\n\n"] {
        for width in [1usize, 3, 20] {
            assert_eq!(
                display_row_count(text, width),
                wrapped_lines_for_display(text, width).len(),
                "text={text:?} width={width}"
            );
        }
    }
}

#[test]
fn newline_selection_extends_to_line_end() {
    let cells = selection_highlight_cells("hi\nthere", Some((2, 3)), 10);
    assert!(cells.contains(&(0, 2)));
    assert!(cells.contains(&(0, 9)));
    assert!(!cells.contains(&(1, 0)));
}

#[test]
fn regular_selection_does_not_extend_past_chars() {
    let cells = selection_highlight_cells("hi\nthere", Some((0, 2)), 10);
    assert_eq!(cells, BTreeSet::from([(0, 0), (0, 1)]));
}

#[test]
fn wrapped_line_count_respects_hard_wrap_and_newline() {
    assert_eq!(wrapped_line_count("", 3), 0);
    assert_eq!(wrapped_line_count("abcd", 3), 2);
    assert_eq!(wrapped_line_count("ab\ncd", 3), 2);
}

#[test]
fn cursor_position_at_end_of_wrapped_text() {
    assert_eq!(cursor_display_position("abcd", 4, 3), (1, 1));
}
