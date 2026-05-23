//! Pure scroll-offset math for a fixed-height viewport over wrapped lines.

/// Largest valid `scroll_offset` (0 when content fits in the viewport).
pub fn max_scroll_offset(total_lines: usize, viewport_height: usize) -> usize {
    if viewport_height == 0 {
        return 0;
    }
    total_lines.saturating_sub(viewport_height)
}

/// Clamp `scroll_offset` so the viewport stays within content bounds.
pub fn clamp_scroll_offset(offset: usize, total_lines: usize, viewport_height: usize) -> usize {
    offset.min(max_scroll_offset(total_lines, viewport_height))
}

/// Scroll up by one line.
pub fn scroll_line_up(offset: usize) -> usize {
    offset.saturating_sub(1)
}

/// Scroll down by one line.
pub fn scroll_line_down(offset: usize, total_lines: usize, viewport_height: usize) -> usize {
    (offset + 1).min(max_scroll_offset(total_lines, viewport_height))
}

/// Pin the viewport to the last lines (e.g. while streaming new tokens).
pub fn stick_to_bottom(total_lines: usize, viewport_height: usize) -> usize {
    max_scroll_offset(total_lines, viewport_height)
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
    if total_lines == 0 || viewport_height == 0 {
        return 0..0;
    }
    let start = scroll_offset.min(total_lines.saturating_sub(1));
    let end = (start + viewport_height).min(total_lines);
    start..end
}
