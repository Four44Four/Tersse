//! Clip rectangles and text to a terminal window (0-indexed max row/column).

use crate::pure::scroll_view;

/// Max row index usable for TUI elements (`get_max_y()` minus one reserved row).
pub fn content_max_y(terminal_max_y: i32) -> i32 {
    (terminal_max_y - 1).max(0)
}

/// Columns visible from `x` while reserving the rightmost terminal column.
pub fn cols_visible_from(x: i32, max_x: i32) -> i32 {
    if x > max_x {
        0
    } else {
        (max_x - x).max(0)
    }
}

/// Rows visible from `y` through the usable bottom row (`terminal_max_y` minus one).
///
/// When `y` is above the viewport (negative), returns the number of usable rows from
/// terminal row 0 downward so partially scrolled content can be clipped correctly.
pub fn rows_visible_from(y: i32, terminal_max_y: i32) -> i32 {
    let max_y = content_max_y(terminal_max_y);
    if y > max_y {
        0
    } else if y < 0 {
        max_y + 1
    } else {
        max_y - y + 1
    }
}

/// Max characters that can be written on `row_y` without ncurses auto-wrap.
///
/// The rightmost terminal column is reserved on all usable rows.

/// Line indices `[start, end)` within an element that intersect visible terminal rows.
pub fn visible_element_line_range(
    anchor_y: i32,
    logical_rows: i32,
    terminal_max_y: i32,
) -> std::ops::Range<i32> {
    if logical_rows <= 0 {
        return 0..0;
    }
    let max_row = content_max_y(terminal_max_y);
    let first = if anchor_y < 0 { -anchor_y } else { 0 };
    let last = (max_row - anchor_y).min(logical_rows - 1);
    if first > last {
        0..0
    } else {
        first..last + 1
    }
}

/// Whether `row_y` is within the usable row range for TUI elements.
pub fn row_is_visible(row_y: i32, terminal_max_y: i32) -> bool {
    let max_y = content_max_y(terminal_max_y);
    (0..=max_y).contains(&row_y)
}

pub fn cols_for_printing(x: i32, max_x: i32, row_y: i32, terminal_max_y: i32) -> i32 {
    if x > max_x || !row_is_visible(row_y, terminal_max_y) {
        0
    } else {
        (max_x - x).max(0)
    }
}

/// Max printable columns for one row of an element at `(x, row_y)` with `element_width`.
pub fn max_element_row_cols(
    x: i32,
    max_x: i32,
    row_y: i32,
    terminal_max_y: i32,
    element_width: i32,
) -> i32 {
    cols_for_printing(x, max_x, row_y, terminal_max_y).min(element_width.max(0))
}

/// Truncate `height` so the rectangle ending at `y + height - 1` does not extend past usable height.
pub fn clip_height_at_terminal(y: i32, height: i32, terminal_max_y: i32) -> i32 {
    rows_visible_from(y, terminal_max_y).min(height).max(0)
}

/// Counts terminal rows in `[start_y, start_y + logical_rows)` that are visible and not blocked.
pub fn drawable_rows_in_span(
    start_y: i32,
    logical_rows: i32,
    terminal_max_y: i32,
    mut is_blocked_row: impl FnMut(i32) -> bool,
) -> usize {
    if logical_rows <= 0 {
        return 0;
    }
    let end_y = start_y.saturating_add(logical_rows);
    let mut count = 0usize;
    for screen_y in start_y..end_y {
        if row_is_visible(screen_y, terminal_max_y) && !is_blocked_row(screen_y) {
            count += 1;
        }
    }
    count
}

/// Wrapped line indices to draw for a text field at `anchor_screen_y`.
///
/// Intersects in-field scroll range with the lines that actually intersect the terminal.
pub fn text_input_draw_line_indices(
    anchor_screen_y: i32,
    total_lines: usize,
    scroll_offset: usize,
    scroll_viewport: usize,
    terminal_max_y: i32,
) -> std::ops::Range<usize> {
    if total_lines == 0 || scroll_viewport == 0 {
        return 0..0;
    }
    let viewport_rows =
        visible_element_line_range(anchor_screen_y, scroll_viewport as i32, terminal_max_y);
    if viewport_rows.is_empty() {
        return 0..0;
    }
    let scroll_range = scroll_view::visible_line_range(scroll_offset, scroll_viewport, total_lines);
    let start = scroll_range
        .start
        .saturating_add(viewport_rows.start as usize);
    let end = scroll_range
        .start
        .saturating_add(viewport_rows.end as usize)
        .min(scroll_range.end)
        .min(total_lines);
    if start >= end {
        0..0
    } else {
        start..end
    }
}

/// Clip a `(width, height)` rectangle anchored at `(x, y)` to fit the terminal.
pub fn clip_rect(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    max_x: i32,
    terminal_max_y: i32,
) -> (i32, i32) {
    let w = width.min(cols_visible_from(x, max_x)).max(0);
    let h = clip_height_at_terminal(y, height, terminal_max_y);
    if w == 0 || h == 0 {
        (0, 0)
    } else {
        (w, h)
    }
}

/// Truncate `text` so it occupies at most `max_cols` terminal columns.
pub fn clip_str_to_cols(text: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        String::new()
    } else {
        text.chars().take(max_cols).collect()
    }
}
