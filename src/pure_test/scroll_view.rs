//! Scroll-view helpers for integration tests.

/// Largest valid `scroll_offset` (0 when content fits in the viewport).
pub fn max_scroll_offset(total_lines: usize, viewport_height: usize) -> usize {
    crate::pure::scroll_view::max_scroll_offset(total_lines, viewport_height)
}

/// Clamp `scroll_offset` so the viewport stays within content bounds.
pub fn clamp_scroll_offset(offset: usize, total_lines: usize, viewport_height: usize) -> usize {
    crate::pure::scroll_view::clamp_scroll_offset(offset, total_lines, viewport_height)
}

/// Scroll up by one line.
pub fn scroll_line_up(offset: usize) -> usize {
    crate::pure::scroll_view::scroll_line_up(offset)
}

/// Scroll down by one line.
pub fn scroll_line_down(offset: usize, total_lines: usize, viewport_height: usize) -> usize {
    crate::pure::scroll_view::scroll_line_down(offset, total_lines, viewport_height)
}

/// Pin the viewport to the last lines (e.g. while streaming new tokens).
pub fn stick_to_bottom(total_lines: usize, viewport_height: usize) -> usize {
    crate::pure::scroll_view::max_scroll_offset(total_lines, viewport_height)
}

/// True when wrapped content exceeds the visible viewport.
pub fn content_overflows(total_lines: usize, viewport_height: usize) -> bool {
    viewport_height > 0 && total_lines > viewport_height
}

/// Indices of wrapped lines to render for the current scroll position.
pub fn visible_line_range(
    scroll_offset: usize,
    viewport_height: usize,
    total_lines: usize,
) -> std::ops::Range<usize> {
    crate::pure::scroll_view::visible_line_range(scroll_offset, viewport_height, total_lines)
}
