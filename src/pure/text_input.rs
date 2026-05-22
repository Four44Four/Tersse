//! Pure logic for single-line text input fields (cursor, selection, insert, delete).

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
