//! Word-wrap and caret-to-cell mapping for multi-line text display.

/// Split `text` into display lines of at most `width` characters (hard wrap; `\n` starts a new line).
pub fn wrapped_lines(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
            continue;
        }
        current.push(ch);
        if current.chars().count() >= width {
            lines.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Number of wrapped lines (0 when `text` is empty).
pub fn wrapped_line_count(text: &str, width: usize) -> usize {
    wrapped_lines(text, width).len()
}

/// Rows to allocate in the UI (at least one row when empty).
pub fn display_row_count(text: &str, width: usize) -> usize {
    if text.is_empty() {
        1
    } else {
        wrapped_line_count(text, width).max(1)
    }
}

/// Map a character-index caret to `(line, col)` in the wrapped display.
pub fn cursor_display_position(text: &str, cursor: usize, width: usize) -> (usize, usize) {
    let width = width.max(1);
    let mut line = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in text.chars() {
        if idx == cursor {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
            if col >= width {
                line += 1;
                col = 0;
            }
        }
        idx += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
