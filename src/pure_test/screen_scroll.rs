//! Screen-scroll helpers for integration tests.

/// Total content height in rows from the top of the screen (0-based exclusive end).
pub fn screen_content_height(element_spans: &[(u16, usize)]) -> usize {
    crate::pure::screen_scroll::screen_content_height(element_spans)
}

/// Usable terminal rows available for drawing TUI content.
pub fn screen_viewport_height(terminal_max_y: i32) -> usize {
    crate::pure::screen_scroll::screen_viewport_height(terminal_max_y)
}

/// Clamp a screen scroll offset to valid bounds.
pub fn clamp_screen_scroll(offset: usize, content_height: usize, viewport_height: usize) -> usize {
    crate::pure::screen_scroll::clamp_screen_scroll(offset, content_height, viewport_height)
}

/// Scroll the screen up by one row (decrease offset).
pub fn scroll_screen_up(offset: usize) -> usize {
    crate::pure::screen_scroll::scroll_screen_up(offset)
}

/// Scroll the screen down by one row (increase offset).
pub fn scroll_screen_down(offset: usize, content_height: usize, viewport_height: usize) -> usize {
    crate::pure::scroll_view::scroll_line_down(offset, content_height, viewport_height)
}

/// Map a logical row to the row drawn in the terminal after applying screen scroll.
pub fn apply_scroll_to_y(logical_y: i32, scroll_offset: usize) -> i32 {
    logical_y - scroll_offset as i32
}

/// Screen-scroll offset that keeps `focus_row` (a logical document row) inside the viewport.
pub fn screen_scroll_to_show_row(
    focus_row: usize,
    content_height: usize,
    viewport_height: usize,
) -> usize {
    crate::pure::scroll_view::clamp_scroll_offset(
        focus_row.saturating_sub(viewport_height.saturating_sub(1)),
        content_height,
        viewport_height,
    )
}

/// Screen-scroll offset that keeps `cursor_row` visible.
pub fn screen_scroll_to_show_cursor_row(
    cursor_row: usize,
    current_scroll: usize,
    content_height: usize,
    viewport_height: usize,
    viewport_shift: i32,
) -> usize {
    crate::pure::screen_scroll::screen_scroll_to_show_cursor_row(
        cursor_row,
        current_scroll,
        content_height,
        viewport_height,
        viewport_shift,
    )
}
