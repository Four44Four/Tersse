use crate::pure::text_wrap;

/// Whether wrapped display rows fit within `max_rows`.
pub fn state_fits_in_max_rows(state: &TextInputState, width: usize, max_rows: usize) -> bool {
    text_wrap::display_row_count(&state.text, width) <= max_rows.max(1)
}

/// Truncate `text` so its wrapped row count does not exceed `max_rows`.
pub fn truncate_text_to_max_rows(text: &str, width: usize, max_rows: usize) -> String {
    let max_rows = max_rows.max(1);
    let mut out = String::new();
    for ch in text.chars() {
        let mut trial = out.clone();
        trial.push(ch);
        if text_wrap::display_row_count(&trial, width) > max_rows {
            break;
        }
        out.push(ch);
    }
    out
}

/// Clamp text and cursor so the field does not exceed `max_rows` display lines.
pub fn clamp_state_to_max_rows(
    state: &TextInputState,
    width: usize,
    max_rows: usize,
) -> TextInputState {
    if state_fits_in_max_rows(state, width, max_rows) {
        return state.clone();
    }
    let text = truncate_text_to_max_rows(&state.text, width, max_rows);
    let cursor = state.cursor.min(text.chars().count());
    TextInputState {
        text,
        cursor,
        selection_anchor: None,
    }
}

/// Insert a string at the cursor (replaces any active selection).
pub fn insert_text(state: &TextInputState, text: &str) -> Option<TextInputState> {
    crate::pure::text_input::paste_text(state, text)
}

pub use crate::pure::text_input::{
    backspace, clear_selection, copy_selection, cursor_left, cursor_right, cut_selection,
    delete_forward, delete_selection, insert_char, insert_newline, insert_tab, selection_range,
    selection_text, TextInputState,
};

pub fn paste_text(state: &TextInputState, paste: &str) -> Option<TextInputState> {
    crate::pure::text_input::paste_text(state, paste)
}
