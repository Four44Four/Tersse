//! Global crate constants.

/// Milliseconds to wait after the last terminal resize before redrawing the screen.
pub const TERM_RESIZE_DEBOUNCE_MS: u64 = 500;

/// Milliseconds to wait after the last UI refresh request before redrawing the screen.
pub const UI_REDRAW_DEBOUNCE_MS: u64 = 20;
