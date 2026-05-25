//! Global crate constants.

use crate::Color;

/// Milliseconds to wait after the last terminal resize before redrawing the screen.
pub const TERM_RESIZE_DEBOUNCE_MS: u64 = 500;

/// Which screen edge the message gutter occupies when visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MsgGutterSide {
    #[default]
    Top,
    Bottom,
}

/// Background color for the message gutter overlay.
pub const MSG_GUTTER_BG_COLOR: Color = Color::Black;

/// Screen edge where the message gutter is drawn.
pub const MSG_GUTTER_SIDE: MsgGutterSide = MsgGutterSide::Bottom;

/// Maximum number of terminal rows the message gutter may occupy.
pub const MSG_GUTTER_MAX_HEIGHT: usize = 5;

/// Milliseconds before a displayed message gutter hides itself.
pub const MSG_GUTTER_DURA_MS: u64 = 5000;

/// Suffix shown when a new message arrives before the previous one expired.
pub const MSG_GUTTER_MULTI_MSG_STR: &str = "[+]";

/// Foreground color for [`MSG_GUTTER_MULTI_MSG_STR`].
pub const MSG_GUTTER_MULTI_MSG_COLOR: Color = Color::Cyan;

/// Milliseconds to wait between UI session queue update driven element redraws.
pub const UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS: u64 = 20;

/// Idle gap that ends a terminal input burst when coalescing paste/key runs.
pub const TERMINAL_POLL_COALESCE_IDLE_MS: u64 = 2;

/// Debug placeholder fill color for existing-element draw/redraw visualization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DebugDrawDelayColor {
    Red,
}

/// When `true`, element draw/redraw paths (including fullscreen [`RuntimeUi::draw`])
/// show a solid placeholder rectangle before drawing the real element.
///
/// Enabled by the `debug_should_draw_do_delay` Cargo feature.
#[allow(dead_code)]
pub const DEBUG_SHOULD_DRAW_DO_DELAY: bool = cfg!(feature = "debug_should_draw_do_delay");

/// Placeholder rectangle background color when [`DEBUG_SHOULD_DRAW_DO_DELAY`] is `true`.
#[allow(dead_code)]
pub const DEBUG_DRAW_DELAY_COLOR: DebugDrawDelayColor = DebugDrawDelayColor::Red;

/// Milliseconds to display the debug placeholder before drawing the real element.
#[allow(dead_code)]
pub const DEBUG_DRAW_DELAY_MS: u64 = 500;
