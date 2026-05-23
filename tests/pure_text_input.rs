use tersse::pure::text_input::{
    backspace, clamp_state_to_max_rows, copy_selection, cursor_left, cursor_right, cut_selection,
    insert_char, insert_newline, paste_text, selection_range, state_fits_in_max_rows,
    truncate_text_to_max_rows, TextInputState,
};

fn state(text: &str, cursor: usize, anchor: Option<usize>) -> TextInputState {
    TextInputState {
        text: text.to_string(),
        cursor,
        selection_anchor: anchor,
    }
}

#[test]
fn selection_range_orders_anchor_and_cursor() {
    assert_eq!(selection_range(&state("abcd", 1, Some(3))), Some((1, 3)));
    assert_eq!(selection_range(&state("abcd", 3, Some(1))), Some((1, 3)));
}

#[test]
fn insert_replaces_selection() {
    let s = state("hello", 3, Some(1));
    let next = insert_char(&s, 'X').unwrap();
    assert_eq!(next.text, "hXlo");
    assert_eq!(next.cursor, 2);
    assert_eq!(next.selection_anchor, None);
}

#[test]
fn backspace_deletes_selection_only() {
    let s = state("hello", 4, Some(1));
    let next = backspace(&s).unwrap();
    assert_eq!(next.text, "ho");
    assert_eq!(next.cursor, 1);
}

#[test]
fn shift_left_extends_selection() {
    let s = state("abc", 2, None);
    let next = cursor_left(&s, true).unwrap();
    assert_eq!(next.cursor, 1);
    assert_eq!(next.selection_anchor, Some(2));
}

#[test]
fn copy_clears_selection() {
    let s = state("hello", 4, Some(1));
    let (next, text) = copy_selection(&s).unwrap();
    assert_eq!(text, "ell");
    assert_eq!(next.text, "hello");
    assert_eq!(next.selection_anchor, None);
}

#[test]
fn cut_removes_selection() {
    let s = state("hello", 4, Some(1));
    let (next, text) = cut_selection(&s).unwrap();
    assert_eq!(text, "ell");
    assert_eq!(next.text, "ho");
    assert_eq!(next.cursor, 1);
}

#[test]
fn paste_replaces_selection() {
    let s = state("hello", 4, Some(1));
    let next = paste_text(&s, "X").unwrap();
    assert_eq!(next.text, "hXo");
    assert_eq!(next.cursor, 2);
}

#[test]
fn insert_newline_splits_text() {
    let s = state("helloworld", 5, None);
    let next = insert_newline(&s).unwrap();
    assert_eq!(next.text, "hello\nworld");
    assert_eq!(next.cursor, 6);
}

#[test]
fn insert_newline_replaces_selection() {
    let s = state("hello", 4, Some(1));
    let next = insert_newline(&s).unwrap();
    assert_eq!(next.text, "h\no");
    assert_eq!(next.cursor, 2);
}

#[test]
fn paste_includes_newlines() {
    let s = state("ab", 1, None);
    let next = paste_text(&s, "X\nY").unwrap();
    assert_eq!(next.text, "aX\nYb");
    assert_eq!(next.cursor, 4);
}

#[test]
fn arrow_at_text_edges_does_nothing() {
    let left_edge = state("abc", 0, None);
    let right_edge = state("abc", 3, None);
    assert_eq!(cursor_left(&left_edge, false), None);
    assert_eq!(cursor_right(&right_edge, false), None);
}

#[test]
fn horizontal_arrow_without_shift_clears_selection() {
    let s = state("abc", 2, Some(0));
    let next = cursor_left(&s, false).unwrap();
    assert_eq!(next.cursor, 1);
    assert_eq!(next.selection_anchor, None);
}

#[test]
fn control_character_insert_is_rejected() {
    let s = state("abc", 1, None);
    assert_eq!(insert_char(&s, '\u{0007}'), None);
}

#[test]
fn truncate_text_to_max_rows_stops_before_overflow() {
    let text = "abcdef";
    assert_eq!(truncate_text_to_max_rows(&text, 3, 1), "ab");
    assert_eq!(truncate_text_to_max_rows(&text, 3, 2), "abcde");
}

#[test]
fn clamp_state_to_max_rows_trims_paste_overflow() {
    let s = state("abc", 3, None);
    let pasted = paste_text(&s, "defghi").unwrap();
    assert!(!state_fits_in_max_rows(&pasted, 3, 2));
    let clamped = clamp_state_to_max_rows(&pasted, 3, 2);
    assert_eq!(clamped.text, "abcde");
    assert!(state_fits_in_max_rows(&clamped, 3, 2));
}
