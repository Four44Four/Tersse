//! Global crate constants.

/// Milliseconds to wait after the last terminal resize before redrawing the screen.
pub const TERM_RESIZE_DEBOUNCE_MS: u64 = 500;

/// Milliseconds to wait between UI session queue update driven element redraws.
pub const UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS: u64 = 20;

/// Debug placeholder fill color for existing-element draw/redraw visualization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DebugDrawDelayColor {
    Red,
}

/// When `true`, element draw/redraw paths (including fullscreen [`RuntimeUi::draw`])
/// show a solid placeholder rectangle before drawing the real element. Requires a rebuild
/// after changing this value (`build.rs` reads it at compile time).
#[allow(dead_code)]
pub const DEBUG_SHOULD_DRAW_DO_DELAY: bool = false;

/// Placeholder rectangle background color when [`DEBUG_SHOULD_DRAW_DO_DELAY`] is `true`.
#[allow(dead_code)]
pub const DEBUG_DRAW_DELAY_COLOR: DebugDrawDelayColor = DebugDrawDelayColor::Red;

/// Milliseconds to display the debug placeholder before drawing the real element.
#[allow(dead_code)]
pub const DEBUG_DRAW_DELAY_MS: u64 = 500;
