//! Pure logic for single-line text input fields (cursor, insert, delete).

/// Snapshot of a one-line text field and caret position (character indices).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TextInputState {
    pub text: String,
    pub cursor: usize,
}

impl TextInputState {
    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }
}

fn byte_index_for_char(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
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

/// Insert a printable character at the cursor. Returns `None` for control characters.
pub fn insert_char(state: &TextInputState, c: char) -> Option<TextInputState> {
    if c.is_control() {
        return None;
    }
    let mut text = state.text.clone();
    insert_char_at(&mut text, state.cursor, c);
    Some(TextInputState {
        text,
        cursor: state.cursor + 1,
    })
}

/// Remove the character before the cursor (Backspace).
pub fn backspace(state: &TextInputState) -> Option<TextInputState> {
    let mut text = state.text.clone();
    if !remove_char_before(&mut text, state.cursor) {
        return None;
    }
    Some(TextInputState {
        text,
        cursor: state.cursor - 1,
    })
}

/// Remove the character at the cursor (Delete).
pub fn delete_forward(state: &TextInputState) -> Option<TextInputState> {
    let mut text = state.text.clone();
    if !remove_char_at(&mut text, state.cursor) {
        return None;
    }
    Some(TextInputState {
        text,
        cursor: state.cursor,
    })
}

/// Move the cursor one character left.
pub fn cursor_left(state: &TextInputState) -> Option<TextInputState> {
    if state.cursor == 0 {
        return None;
    }
    Some(TextInputState {
        text: state.text.clone(),
        cursor: state.cursor - 1,
    })
}

/// Move the cursor one character right.
pub fn cursor_right(state: &TextInputState) -> Option<TextInputState> {
    if state.cursor >= state.char_len() {
        return None;
    }
    Some(TextInputState {
        text: state.text.clone(),
        cursor: state.cursor + 1,
    })
}

/// Insert a literal tab at the cursor.
pub fn insert_tab(state: &TextInputState) -> Option<TextInputState> {
    insert_char(state, '\t')
}
