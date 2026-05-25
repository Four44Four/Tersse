pub use crate::pure::terminal_bounds::{
    clip_height_at_terminal, clip_rect, clip_str_to_cols, cols_for_printing, cols_visible_from,
    content_max_y, drawable_rows_in_span, element_intersects_terminal_viewport,
    max_element_row_cols, row_is_visible, rows_visible_from, visible_element_line_range,
};

use crate::pure::scroll_view;

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
