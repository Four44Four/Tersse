//! Word-wrap, caret-to-cell mapping, and selection highlighting for multi-line text display.

use std::collections::BTreeSet;

/// Hard-wrap a single logical line (no `\n` characters).
fn wrap_logical_line(segment: &str, width: usize) -> Vec<String> {
    if segment.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in segment.chars() {
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

/// Split `text` into display lines of at most `width` characters (hard wrap; `\n` starts a new line).
pub fn wrapped_lines(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if text.is_empty() {
        return Vec::new();
    }
    text.split('\n')
        .flat_map(|segment| wrap_logical_line(segment, width))
        .collect()
}

/// Wrapped lines for rendering and scrolling; empty `text` yields one blank line.
///
/// Line count matches [`display_row_count`], unlike [`wrapped_lines`] alone.
pub fn wrapped_lines_for_display(text: &str, width: usize) -> Vec<String> {
    let mut lines = wrapped_lines(text, width);
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Rows to allocate in the UI (matches [`wrapped_lines_for_display`] line count).
pub fn display_row_count(text: &str, width: usize) -> usize {
    wrapped_lines_for_display(text, width).len()
}

/// Display rows consumed by a logical line that ends at a `\n` (matches [`wrap_logical_line`]).
fn logical_line_display_rows(seg_line: usize, col: usize) -> usize {
    if col == 0 && seg_line == 0 {
        1
    } else if col == 0 {
        seg_line
    } else {
        seg_line + 1
    }
}

/// Map caret index to `(line, col)` aligned with [`wrapped_lines`].
fn caret_cell(
    line_base: usize,
    seg_line: usize,
    col: usize,
    cursor_idx: usize,
    text: &str,
    width: usize,
) -> (usize, usize) {
    let at_segment_end = text
        .chars()
        .nth(cursor_idx)
        .is_some_and(|c| c == '\n')
        || cursor_idx >= text.chars().count();
    if col == 0 && seg_line > 0 && at_segment_end {
        // Caret after the last character on a full visual row (newline has no width).
        (line_base + seg_line - 1, width)
    } else {
        (line_base + seg_line, col)
    }
}

/// Map a character-index caret to `(line, col)` in the wrapped display.
pub fn cursor_display_position(text: &str, cursor: usize, width: usize) -> (usize, usize) {
    let width = width.max(1);
    let mut line_base = 0usize;
    let mut seg_line = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in text.chars() {
        if idx == cursor {
            return caret_cell(line_base, seg_line, col, idx, text, width);
        }
        if ch == '\n' {
            line_base += logical_line_display_rows(seg_line, col);
            seg_line = 0;
            col = 0;
        } else {
            col += 1;
            if col >= width {
                seg_line += 1;
                col = 0;
            }
        }
        idx += 1;
    }
    caret_cell(line_base, seg_line, col, idx, text, width)
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
