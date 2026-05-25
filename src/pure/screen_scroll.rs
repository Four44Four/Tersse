//! Pure helpers for scrolling the entire TUI screen within the terminal viewport.

use crate::pure::scroll_view;
use crate::pure::terminal_bounds;

/// Total content height in rows from the top of the screen (0-based exclusive end).
///
/// `element_spans` lists each element's anchor row and logical height in rows.
pub fn screen_content_height(element_spans: &[(u16, usize)]) -> usize {
    let mut height = 0;
    for &(y, h) in element_spans {
        height = height.max(y as usize + h);
    }
    height
}

/// Usable terminal rows available for drawing TUI content.
pub fn screen_viewport_height(terminal_max_y: i32) -> usize {
    (terminal_bounds::content_max_y(terminal_max_y) + 1) as usize
}

/// Clamp a screen scroll offset to valid bounds.
pub fn clamp_screen_scroll(offset: usize, content_height: usize, viewport_height: usize) -> usize {
    scroll_view::clamp_scroll_offset(offset, content_height, viewport_height)
}

/// Scroll the screen up by one row (decrease offset).
pub fn scroll_screen_up(offset: usize) -> usize {
    scroll_view::scroll_line_up(offset)
}

/// Scroll the screen down by one row (increase offset).
pub fn scroll_screen_down(offset: usize, content_height: usize, viewport_height: usize) -> usize {
    scroll_view::scroll_line_down(offset, content_height, viewport_height)
}

/// Map a logical row to the row drawn in the terminal after applying screen scroll.
///
/// The result may be negative when the element is scrolled above the viewport; callers
/// should skip or clip rows that are not visible.
pub fn apply_scroll_to_y(logical_y: i32, scroll_offset: usize) -> i32 {
    logical_y - scroll_offset as i32
}

/// Screen-scroll offset that keeps `focus_row` (a logical document row) inside the viewport.
pub fn screen_scroll_to_show_row(
    focus_row: usize,
    content_height: usize,
    viewport_height: usize,
) -> usize {
    scroll_view::clamp_scroll_offset(
        focus_row.saturating_sub(viewport_height.saturating_sub(1)),
        content_height,
        viewport_height,
    )
}
