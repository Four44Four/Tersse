//! Pure layout and state helpers for the temporary message gutter overlay.

use crate::pure::scroll_view;
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
    screen_scroll_up_reveal: usize,
    screen_rows: std::ops::Range<i32>,
) -> bool {
    let logical_start = element_y as i32;
    let logical_end = logical_start + element_height.max(1) as i32;
    for screen_y in screen_rows {
        let logical_y =
            screen_y + screen_scroll as i32 - screen_scroll_up_reveal as i32;
        if logical_y >= logical_start && logical_y < logical_end {
            return true;
        }
    }
    false
}

/// Viewport row count available for scrolling content while the gutter is visible.
///
/// The message gutter is fixed screen space at the top or bottom edge. Those rows are not usable
/// for content, so the scrollable viewport is shorter by the gutter height.
pub fn viewport_height_for_screen_scroll(
    full_viewport_height: usize,
    gutter_visible: bool,
    gutter_height: usize,
    _side: MsgGutterSide,
) -> usize {
    if gutter_visible && gutter_height > 0 {
        full_viewport_height.saturating_sub(gutter_height).max(1)
    } else {
        full_viewport_height
    }
}

/// Viewport height used to clamp downward screen scroll (full height when the gutter is on top).
pub fn viewport_height_for_down_scroll_clamp(
    full_viewport_height: usize,
    gutter_visible: bool,
    gutter_height: usize,
    side: MsgGutterSide,
) -> usize {
    match side {
        MsgGutterSide::Bottom => {
            viewport_height_for_screen_scroll(full_viewport_height, gutter_visible, gutter_height, side)
        }
        MsgGutterSide::Top => full_viewport_height,
    }
}

/// Extra scroll rows allowed to reveal content hidden under the gutter.
pub fn gutter_reveal_rows(
    base_content_height: usize,
    full_viewport_height: usize,
    gutter_visible: bool,
    gutter_height: usize,
    side: MsgGutterSide,
) -> usize {
    if !gutter_visible || gutter_height == 0 {
        return 0;
    }
    let minimum_bonus = gutter_height.min(full_viewport_height);
    let effective = viewport_height_for_screen_scroll(
        full_viewport_height,
        true,
        gutter_height,
        side,
    );
    let reveal_at = scroll_view::max_scroll_offset(base_content_height, effective);
    let base_max = scroll_view::max_scroll_offset(base_content_height, full_viewport_height);
    reveal_at.saturating_sub(base_max).max(minimum_bonus)
}

/// Largest valid screen scroll offset given base content and gutter/reveal state.
///
/// After a scroll-reveal hide, `reveal_scroll_cap` limits re-entry into the bonus scroll range
/// but never below the normal content maximum for the full terminal viewport.
pub fn max_screen_scroll_offset(
    base_content_height: usize,
    viewport_height: usize,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    let base_max = scroll_view::max_scroll_offset(base_content_height, viewport_height);
    match reveal_scroll_cap {
        Some(cap) => cap.max(base_max),
        None => base_max,
    }
}

/// Clamp a screen scroll offset using [`max_screen_scroll_offset`].
pub fn clamp_screen_scroll_with_gutter(
    offset: usize,
    base_content_height: usize,
    viewport_height: usize,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    let max = max_screen_scroll_offset(base_content_height, viewport_height, reveal_scroll_cap);
    offset.min(max)
}

/// Scroll the screen down by one row toward [`max_screen_scroll_offset`].
pub fn scroll_screen_down_with_gutter(
    offset: usize,
    base_content_height: usize,
    viewport_height: usize,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    let max = max_screen_scroll_offset(base_content_height, viewport_height, reveal_scroll_cap);
    (offset + 1).min(max)
}

/// Largest valid upward reveal offset while the top gutter is visible or capped after hide.
pub fn max_screen_scroll_up_reveal(
    base_content_height: usize,
    full_viewport_height: usize,
    gutter_visible: bool,
    gutter_height: usize,
    side: MsgGutterSide,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    if !matches!(side, MsgGutterSide::Top) {
        return 0;
    }
    let bonus = gutter_reveal_rows(
        base_content_height,
        full_viewport_height,
        gutter_visible,
        gutter_height,
        side,
    );
    if gutter_visible {
        return bonus;
    }
    reveal_scroll_cap.unwrap_or(0)
}

/// Clamp upward reveal offset using [`max_screen_scroll_up_reveal`].
pub fn clamp_screen_scroll_up_reveal_with_gutter(
    up_reveal: usize,
    base_content_height: usize,
    full_viewport_height: usize,
    gutter_visible: bool,
    gutter_height: usize,
    side: MsgGutterSide,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    let max = max_screen_scroll_up_reveal(
        base_content_height,
        full_viewport_height,
        gutter_visible,
        gutter_height,
        side,
        reveal_scroll_cap,
    );
    up_reveal.min(max)
}

/// Scroll the screen up by one row into the top-gutter reveal range.
pub fn scroll_screen_up_with_gutter(
    up_reveal: usize,
    base_content_height: usize,
    full_viewport_height: usize,
    gutter_height: usize,
    reveal_scroll_cap: Option<usize>,
) -> usize {
    let max = max_screen_scroll_up_reveal(
        base_content_height,
        full_viewport_height,
        true,
        gutter_height,
        MsgGutterSide::Top,
        reveal_scroll_cap,
    );
    (up_reveal + 1).min(max)
}

/// Ratchet the post-reveal bonus scroll ceiling down when the user scrolls up.
///
/// Scrolling up within normal content (`screen_scroll <= base_max`) clamps the cap to
/// `base_max` so screen scroll is never permanently limited below the content bottom.
pub fn ratchet_gutter_scroll_cap_on_up(
    reveal_scroll_cap: Option<usize>,
    screen_scroll: usize,
    base_max_scroll: usize,
) -> Option<usize> {
    reveal_scroll_cap.map(|cap| {
        if screen_scroll <= base_max_scroll {
            cap.min(base_max_scroll)
        } else {
            cap.min(screen_scroll)
        }
    })
}

/// Ratchet the post-reveal top bonus ceiling down when the user scrolls down.
///
/// Leaving the upward reveal range (`screen_scroll_up_reveal == 0`) clears the cap so the
/// bonus top padding cannot be entered again until the gutter covers the screen again.
pub fn ratchet_gutter_up_reveal_cap_on_down(
    reveal_scroll_cap: Option<usize>,
    screen_scroll_up_reveal: usize,
) -> Option<usize> {
    reveal_scroll_cap.map(|cap| {
        if screen_scroll_up_reveal == 0 {
            0
        } else {
            cap.min(screen_scroll_up_reveal)
        }
    })
}

/// Whether the viewport shows logical rows past the end of real content.
///
/// While the gutter is visible, padding is never drawn (gutter rows stay fixed screen space).
pub fn screen_scroll_shows_padding(
    screen_scroll: usize,
    base_content_height: usize,
    full_viewport_height: usize,
    gutter_visible: bool,
) -> bool {
    if gutter_visible || full_viewport_height == 0 {
        return false;
    }
    let base_max = scroll_view::max_scroll_offset(base_content_height, full_viewport_height);
    screen_scroll > base_max
}

/// Screen rows that should be filled with default background below content end.
pub fn padding_screen_rows(
    screen_scroll: usize,
    base_content_height: usize,
    terminal_max_y: i32,
) -> std::ops::Range<i32> {
    let max_row = content_max_y(terminal_max_y);
    let first_screen = base_content_height as i32 - screen_scroll as i32;
    let start = first_screen.clamp(0, max_row + 1);
    if start > max_row {
        0..0
    } else {
        start..max_row + 1
    }
}

/// True when the viewport shows default background above the content start after a top reveal hide.
pub fn screen_scroll_shows_top_padding(
    screen_scroll: usize,
    screen_scroll_up_reveal: usize,
    gutter_visible: bool,
) -> bool {
    !gutter_visible && screen_scroll_up_reveal > screen_scroll
}

/// Screen rows that should be filled with default background above content start.
pub fn top_padding_screen_rows(
    screen_scroll: usize,
    screen_scroll_up_reveal: usize,
) -> std::ops::Range<i32> {
    let rows = screen_scroll_up_reveal.saturating_sub(screen_scroll);
    if rows == 0 {
        0..0
    } else {
        0..rows as i32
    }
}

