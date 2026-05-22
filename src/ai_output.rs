//! Scrollable AI response region (wrap at window width, locked-input colors).

use crate::constants::{ai_output_color_pair, COL_BTN};
use crate::pure::{scroll_view, text_wrap};
use pancurses::{COLOR_PAIR, Window};

/// Visible region for AI output below the controls row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiOutputViewport {
    pub start_y: i32,
    pub start_x: i32,
    pub width: usize,
    pub height: usize,
}

impl AiOutputViewport {
    pub fn from_window(win: &Window, response_y: i32) -> Self {
        let (max_y, max_x) = win.get_max_yx();
        Self {
            start_y: response_y,
            start_x: COL_BTN,
            width: (max_x - COL_BTN).max(1) as usize,
            height: (max_y - response_y).max(0) as usize,
        }
    }
}

pub fn wrapped_line_count(text: &str, viewport: AiOutputViewport) -> usize {
    text_wrap::wrapped_line_count(text, viewport.width)
}

pub fn clamp_scroll(scroll_offset: usize, text: &str, viewport: AiOutputViewport) -> usize {
    let total = wrapped_line_count(text, viewport);
    scroll_view::clamp_scroll_offset(scroll_offset, total, viewport.height)
}

pub fn content_overflows(text: &str, viewport: AiOutputViewport) -> bool {
    let total = wrapped_line_count(text, viewport);
    scroll_view::content_overflows(total, viewport.height)
}

pub fn scroll_up(scroll_offset: usize) -> usize {
    scroll_view::scroll_line_up(scroll_offset)
}

pub fn scroll_down(scroll_offset: usize, text: &str, viewport: AiOutputViewport) -> usize {
    let total = wrapped_line_count(text, viewport);
    scroll_view::scroll_line_down(scroll_offset, total, viewport.height)
}

pub fn stick_to_bottom(text: &str, viewport: AiOutputViewport) -> usize {
    let total = wrapped_line_count(text, viewport);
    scroll_view::stick_to_bottom(total, viewport.height)
}

fn fill_viewport(win: &Window, viewport: AiOutputViewport, focused: bool) {
    let pair = ai_output_color_pair(focused);
    win.attron(COLOR_PAIR(pair));
    for row in 0..viewport.height as i32 {
        win.mv(viewport.start_y + row, viewport.start_x);
        for _ in 0..viewport.width {
            let _ = win.addch(' ');
        }
    }
    win.attroff(COLOR_PAIR(pair));
}

/// Draw wrapped AI output lines for the current scroll offset inside the viewport.
pub fn draw_scrollable(
    win: &Window,
    text: &str,
    scroll_offset: usize,
    viewport: AiOutputViewport,
    focused: bool,
) {
    if viewport.height == 0 {
        return;
    }

    fill_viewport(win, viewport, focused);

    let lines = text_wrap::wrapped_lines(text, viewport.width);
    let total = lines.len();
    if total == 0 {
        return;
    }

    let offset = scroll_view::clamp_scroll_offset(scroll_offset, total, viewport.height);
    let range = scroll_view::visible_line_range(offset, viewport.height, total);

    let pair = ai_output_color_pair(focused);
    win.attron(COLOR_PAIR(pair));
    for (row, line_idx) in range.enumerate() {
        win.mv(viewport.start_y + row as i32, viewport.start_x);
        let _ = win.addstr(&lines[line_idx]);
    }
    win.attroff(COLOR_PAIR(pair));
}

/// Draw a single-line status message (e.g. waiting for tokens).
pub fn draw_line(win: &Window, y: i32, text: &str, viewport: AiOutputViewport, focused: bool) {
    fill_viewport(win, viewport, focused);
    let pair = ai_output_color_pair(focused);
    win.attron(COLOR_PAIR(pair));
    win.mv(y, viewport.start_x);
    let _ = win.addstr(text);
    win.attroff(COLOR_PAIR(pair));
}
