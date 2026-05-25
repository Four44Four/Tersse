/// Number of wrapped lines (0 when `text` is empty).
pub fn wrapped_line_count(text: &str, width: usize) -> usize {
    crate::pure::text_wrap::wrapped_lines(text, width).len()
}

pub use crate::pure::text_wrap::{
    cursor_display_position, display_row_count, selection_highlight_cells, wrapped_lines,
    wrapped_lines_for_display,
};
