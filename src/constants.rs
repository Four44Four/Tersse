//! Global crate constants.

/// Default milliseconds to wait when polling the terminal for input.
pub const POLL_TIMEOUT_MS: u64 = 50;

/// Milliseconds to wait after the last terminal resize before redrawing the screen.
pub const TERM_RESIZE_DEBOUNCE_MS: u64 = 500;
