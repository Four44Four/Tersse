//! Clip rectangles and text to a terminal window (0-indexed max row/column).

/// Max row index usable for TUI elements (`get_max_y()` minus one reserved row).
pub fn content_max_y(terminal_max_y: i32) -> i32 {
    (terminal_max_y - 1).max(0)
}

/// Columns visible from `x` through `max_x` inclusive.
pub fn cols_visible_from(x: i32, max_x: i32) -> i32 {
    if x > max_x {
        0
    } else {
        max_x - x + 1
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
/// The lower-right cell `(max_y, max_x)` cannot receive output, so the bottom
/// usable row allows one fewer column than other rows.

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
    let max_y = content_max_y(terminal_max_y);
    if x > max_x || !row_is_visible(row_y, terminal_max_y) {
        0
    } else if row_y == max_y {
        (max_x - x).max(0)
    } else {
        max_x - x + 1
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
