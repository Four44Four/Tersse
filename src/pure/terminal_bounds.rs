//! Clip rectangles and text to a terminal window (0-indexed max row/column).

/// Columns visible from `x` through `max_x` inclusive.
pub fn cols_visible_from(x: i32, max_x: i32) -> i32 {
    if x > max_x {
        0
    } else {
        max_x - x + 1
    }
}

/// Rows visible from `y` through `max_y` inclusive.
pub fn rows_visible_from(y: i32, max_y: i32) -> i32 {
    if y > max_y {
        0
    } else {
        max_y - y + 1
    }
}

/// Max characters that can be written on `row_y` without ncurses auto-wrap.
///
/// The lower-right cell `(max_y, max_x)` cannot receive output, so the bottom
/// terminal row allows one fewer column than other rows.
pub fn cols_for_printing(x: i32, max_x: i32, row_y: i32, max_y: i32) -> i32 {
    if x > max_x {
        0
    } else if row_y >= max_y {
        (max_x - x).max(0)
    } else {
        max_x - x + 1
    }
}

/// Clip a `(width, height)` rectangle anchored at `(x, y)` to fit the terminal.
pub fn clip_rect(x: i32, y: i32, width: i32, height: i32, max_x: i32, max_y: i32) -> (i32, i32) {
    let w = width.min(cols_visible_from(x, max_x)).max(0);
    let h = height.min(rows_visible_from(y, max_y)).max(0);
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
