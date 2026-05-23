//! Pure logic for text input fields (cursor, selection, insert, delete, newlines).

use crate::pure::text_wrap;

/// Snapshot of a one-line text field, caret position, and optional selection anchor.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TextInputState {
    pub text: String,
    pub cursor: usize,
    /// Character index where selection started; active when `Some` and differs from `cursor`.
    pub selection_anchor: Option<usize>,
}

impl TextInputState {
    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn has_selection(&self) -> bool {
        selection_range(self).is_some()
    }
}

/// Inclusive start, exclusive end character indices for the current selection.
pub fn selection_range(state: &TextInputState) -> Option<(usize, usize)> {
    let anchor = state.selection_anchor?;
    if anchor == state.cursor {
        return None;
    }
    let start = anchor.min(state.cursor);
    let end = anchor.max(state.cursor);
    Some((start, end))
}

fn byte_index_for_char(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn remove_range(s: &mut String, start: usize, end: usize) {
    let start_byte = byte_index_for_char(s, start);
    let end_byte = byte_index_for_char(s, end);
    s.drain(start_byte..end_byte);
}

fn insert_char_at(s: &mut String, char_index: usize, c: char) {
    let byte_index = byte_index_for_char(s, char_index);
    s.insert(byte_index, c);
}

fn remove_char_before(s: &mut String, char_index: usize) -> bool {
    if char_index == 0 {
        return false;
    }
    let byte_index = byte_index_for_char(s, char_index - 1);
    let ch_len = s[byte_index..].chars().next().unwrap().len_utf8();
    s.drain(byte_index..byte_index + ch_len);
    true
}

fn remove_char_at(s: &mut String, char_index: usize) -> bool {
    if char_index >= s.chars().count() {
        return false;
    }
    let byte_index = byte_index_for_char(s, char_index);
    let ch_len = s[byte_index..].chars().next().unwrap().len_utf8();
    s.drain(byte_index..byte_index + ch_len);
    true
}

/// Remove the current selection and collapse the caret to its start.
pub fn delete_selection(state: &TextInputState) -> TextInputState {
    let Some((start, end)) = selection_range(state) else {
        return state.clone();
    };
    let mut text = state.text.clone();
    remove_range(&mut text, start, end);
    TextInputState {
        text,
        cursor: start,
        selection_anchor: None,
    }
}

/// Insert a printable character at the cursor. Returns `None` for control characters.
/// Replaces any existing selection.
pub fn insert_char(state: &TextInputState, c: char) -> Option<TextInputState> {
    if c.is_control() {
        return None;
    }
    let base = if state.has_selection() {
        delete_selection(state)
    } else {
        state.clone()
    };
    let mut text = base.text;
    insert_char_at(&mut text, base.cursor, c);
    Some(TextInputState {
        text,
        cursor: base.cursor + 1,
        selection_anchor: None,
    })
}

/// Remove the character before the cursor (Backspace), or the whole selection if active.
pub fn backspace(state: &TextInputState) -> Option<TextInputState> {
    if state.has_selection() {
        return Some(delete_selection(state));
    }
    let mut text = state.text.clone();
    if !remove_char_before(&mut text, state.cursor) {
        return None;
    }
    Some(TextInputState {
        text,
        cursor: state.cursor - 1,
        selection_anchor: None,
    })
}

/// Remove the character at the cursor (Delete), or the whole selection if active.
pub fn delete_forward(state: &TextInputState) -> Option<TextInputState> {
    if state.has_selection() {
        return Some(delete_selection(state));
    }
    let mut text = state.text.clone();
    if !remove_char_at(&mut text, state.cursor) {
        return None;
    }
    Some(TextInputState {
        text,
        cursor: state.cursor,
        selection_anchor: None,
    })
}

/// Move the cursor one character left. With `extend_selection`, grows selection via Shift+Left.
pub fn cursor_left(state: &TextInputState, extend_selection: bool) -> Option<TextInputState> {
    if state.cursor == 0 {
        return None;
    }
    let selection_anchor = if extend_selection {
        state.selection_anchor.or(Some(state.cursor))
    } else {
        None
    };
    Some(TextInputState {
        text: state.text.clone(),
        cursor: state.cursor - 1,
        selection_anchor,
    })
}

/// Move the cursor one character right. With `extend_selection`, grows selection via Shift+Right.
pub fn cursor_right(state: &TextInputState, extend_selection: bool) -> Option<TextInputState> {
    if state.cursor >= state.char_len() {
        return None;
    }
    let selection_anchor = if extend_selection {
        state.selection_anchor.or(Some(state.cursor))
    } else {
        None
    };
    Some(TextInputState {
        text: state.text.clone(),
        cursor: state.cursor + 1,
        selection_anchor,
    })
}

/// Insert a literal tab at the cursor (replaces any selection).
pub fn insert_tab(state: &TextInputState) -> Option<TextInputState> {
    insert_char(state, '\t')
}

/// Insert a line break at the cursor (replaces any selection). Text after the cursor moves to the next line.
pub fn insert_newline(state: &TextInputState) -> Option<TextInputState> {
    let base = if state.has_selection() {
        delete_selection(state)
    } else {
        state.clone()
    };
    let mut text = base.text;
    insert_char_at(&mut text, base.cursor, '\n');
    Some(TextInputState {
        text,
        cursor: base.cursor + 1,
        selection_anchor: None,
    })
}

/// Selected substring, if any.
pub fn selection_text(state: &TextInputState) -> Option<String> {
    let (start, end) = selection_range(state)?;
    let start_byte = byte_index_for_char(&state.text, start);
    let end_byte = byte_index_for_char(&state.text, end);
    Some(state.text[start_byte..end_byte].to_string())
}

/// Clear selection without changing text or cursor.
pub fn clear_selection(state: &TextInputState) -> TextInputState {
    TextInputState {
        text: state.text.clone(),
        cursor: state.cursor,
        selection_anchor: None,
    }
}

/// Copy selection: returns cleared-selection state and text to place on the clipboard.
pub fn copy_selection(state: &TextInputState) -> Option<(TextInputState, String)> {
    let text = selection_text(state)?;
    Some((clear_selection(state), text))
}

/// Cut selection: returns state with selection removed and text for the clipboard.
pub fn cut_selection(state: &TextInputState) -> Option<(TextInputState, String)> {
    let text = selection_text(state)?;
    Some((delete_selection(state), text))
}

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

/// Insert clipboard text at the cursor (replaces any active selection).
pub fn paste_text(state: &TextInputState, paste: &str) -> Option<TextInputState> {
    let base = if state.has_selection() {
        delete_selection(state)
    } else {
        state.clone()
    };
    let mut text = base.text;
    let mut cursor = base.cursor;
    for c in paste
        .chars()
        .filter(|c| !c.is_control() || *c == '\t' || *c == '\n')
    {
        insert_char_at(&mut text, cursor, c);
        cursor += 1;
    }
    Some(TextInputState {
        text,
        cursor,
        selection_anchor: None,
    })
}
