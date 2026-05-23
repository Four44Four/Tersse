//! Word-wrap, caret-to-cell mapping, and selection highlighting for multi-line text display.

use std::collections::BTreeSet;

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
        let (line, _) = cursor_display_position(text, text.chars().count(), width);
        line + 1
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

/// Display cells `(line, col)` that should show the text selection highlight.
/// When a newline in the selection range is included, extends through the rest of that visual line.
pub fn selection_highlight_cells(
    text: &str,
    selection: Option<(usize, usize)>,
    width: usize,
) -> BTreeSet<(usize, usize)> {
    let width = width.max(1);
    let mut cells = BTreeSet::new();
    let Some((start, end)) = selection else {
        return cells;
    };

    let mut line = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in text.chars() {
        if idx >= start && idx < end {
            if ch == '\n' {
                for c in col..width {
                    cells.insert((line, c));
                }
            } else {
                cells.insert((line, col));
            }
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
    cells
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

    #[test]
    fn display_rows_after_explicit_newline() {
        assert_eq!(display_row_count("hello\n", 48), 2);
        assert_eq!(display_row_count("a\nb", 48), 2);
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
}
