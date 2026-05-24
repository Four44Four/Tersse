//! Pure layout and state helpers for the temporary message gutter overlay.

use crate::pure::terminal_bounds::content_max_y;
use crate::pure::text_wrap;

pub use crate::constants::MsgGutterSide;

/// Visible message gutter content.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MessageGutterState {
    pub visible: bool,
    pub message: String,
    pub show_multi_indicator: bool,
    /// Height in terminal rows last used for drawing (for minimal restore).
    pub rendered_height: usize,
}

/// One display row of the message gutter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageGutterLine {
    pub message_text: String,
    pub indicator_text: Option<String>,
}

/// Apply a new message to gutter state.
///
/// When `already_visible` is true, the multi-message indicator is enabled for the new text.
pub fn apply_message(
    state: &MessageGutterState,
    message: impl Into<String>,
    already_visible: bool,
) -> MessageGutterState {
    MessageGutterState {
        visible: true,
        message: message.into(),
        show_multi_indicator: already_visible,
        rendered_height: state.rendered_height,
    }
}

/// Clears gutter visibility while preserving the last rendered height for restore.
pub fn hide_message(state: &MessageGutterState) -> MessageGutterState {
    MessageGutterState {
        visible: false,
        message: String::new(),
        show_multi_indicator: false,
        rendered_height: state.rendered_height,
    }
}

/// Computes gutter height in rows for the current message and terminal width.
pub fn message_gutter_height(
    message: &str,
    show_multi_indicator: bool,
    multi_msg_str: &str,
    terminal_width: usize,
    max_height: usize,
) -> usize {
    layout_message_gutter_lines(message, show_multi_indicator, multi_msg_str, terminal_width)
        .len()
        .min(max_height.max(1))
}

/// Builds wrapped gutter lines, optionally appending the multi-message indicator.
pub fn layout_message_gutter_lines(
    message: &str,
    show_multi_indicator: bool,
    multi_msg_str: &str,
    terminal_width: usize,
) -> Vec<MessageGutterLine> {
    let width = terminal_width.max(1);
    let mut lines: Vec<MessageGutterLine> = if message.is_empty() {
        vec![MessageGutterLine {
            message_text: String::new(),
            indicator_text: None,
        }]
    } else {
        text_wrap::wrapped_lines(message, width)
            .into_iter()
            .map(|message_text| MessageGutterLine {
                message_text,
                indicator_text: None,
            })
            .collect()
    };

    if show_multi_indicator && !multi_msg_str.is_empty() {
        append_multi_message_indicator(&mut lines, multi_msg_str, width);
    }

    if lines.is_empty() {
        lines.push(MessageGutterLine {
            message_text: String::new(),
            indicator_text: None,
        });
    }

    lines
}

fn append_multi_message_indicator(
    lines: &mut Vec<MessageGutterLine>,
    multi_msg_str: &str,
    width: usize,
) {
    let indicator_cols = multi_msg_str.chars().count();
    let last = lines.last_mut().expect("lines is non-empty");
    let last_cols = last.message_text.chars().count();
    if last_cols + 1 + indicator_cols <= width {
        if !last.message_text.is_empty() {
            last.message_text.push(' ');
        }
        last.indicator_text = Some(multi_msg_str.to_string());
    } else {
        lines.push(MessageGutterLine {
            message_text: String::new(),
            indicator_text: Some(multi_msg_str.to_string()),
        });
    }
}

/// Whether printing on `screen_y` can wrap the cursor into the first row of `gutter_rows`.
pub fn row_printing_wraps_into_gutter_block(
    gutter_rows: std::ops::Range<i32>,
    screen_y: i32,
) -> bool {
    !gutter_rows.is_empty() && screen_y + 1 == gutter_rows.start
}

/// Limits printable columns so ncurses does not wrap into the row below.
pub fn clip_cols_to_avoid_wrapping_into_row(
    cols: i32,
    x: i32,
    max_x: i32,
    next_row_is_protected: bool,
) -> i32 {
    if !next_row_is_protected || cols <= 0 {
        return cols;
    }
    cols.min((max_x - x).max(0))
}

/// Returns absolute terminal row indices occupied by the gutter.
pub fn gutter_screen_rows(
    side: MsgGutterSide,
    height: usize,
    terminal_max_y: i32,
) -> std::ops::Range<i32> {
    let height = height.max(1) as i32;
    match side {
        MsgGutterSide::Top => 0..height,
        MsgGutterSide::Bottom => {
            let bottom = content_max_y(terminal_max_y);
            let start = (bottom - height + 1).max(0);
            start..bottom + 1
        }
    }
}

/// Rows that were covered by the previous gutter height but not the new height.
pub fn gutter_rows_to_restore(
    side: MsgGutterSide,
    previous_height: usize,
    new_height: usize,
    terminal_max_y: i32,
) -> std::ops::Range<i32> {
    if previous_height <= new_height {
        return 0..0;
    }
    match side {
        MsgGutterSide::Top => {
            let new_end = new_height.max(1) as i32;
            let prev_end = previous_height as i32;
            new_end..prev_end
        }
        MsgGutterSide::Bottom => {
            let bottom = content_max_y(terminal_max_y);
            let prev_start = (bottom - previous_height as i32 + 1).max(0);
            let new_start = (bottom - new_height.max(1) as i32 + 1).max(0);
            prev_start..new_start
        }
    }
}

/// Whether a logical element row maps to a screen row inside `screen_rows`.
pub fn element_row_intersects_gutter_screen_rows(
    element_y: u16,
    element_height: usize,
    screen_scroll: usize,
    screen_rows: std::ops::Range<i32>,
) -> bool {
    let logical_start = element_y as i32;
    let logical_end = logical_start + element_height.max(1) as i32;
    for screen_y in screen_rows {
        let logical_y = screen_y + screen_scroll as i32;
        if logical_y >= logical_start && logical_y < logical_end {
            return true;
        }
    }
    false
}

/// Whether the screen title occupies any row in `screen_rows` at the current scroll offset.
pub fn title_intersects_gutter_screen_rows(
    has_title: bool,
    screen_scroll: usize,
    screen_rows: std::ops::Range<i32>,
) -> bool {
    if !has_title {
        return false;
    }
    let title_screen_y = 0i32 - screen_scroll as i32;
    screen_rows.contains(&title_screen_y)
}
