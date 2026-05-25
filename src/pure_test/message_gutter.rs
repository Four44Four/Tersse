pub use crate::pure::message_gutter::*;

/// Whether scroll input should hide the message gutter.
///
/// The gutter is only removed when its display duration expires; keyboard scrolling never hides it.
pub fn should_hide_gutter_by_scroll_reveal(
    _screen_scroll: usize,
    _screen_scroll_up_reveal: usize,
    _base_content_height: usize,
    _full_viewport_height: usize,
    _gutter_height: usize,
    _side: crate::pure::message_gutter::MsgGutterSide,
) -> bool {
    false
}
