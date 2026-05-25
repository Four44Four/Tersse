//! Public API for building and styling TUI elements.

pub mod clipboard;
mod constants;
pub mod element_presets;
mod element_id;
#[cfg(not(feature = "pure-tests"))]
mod pure;
#[cfg(feature = "pure-tests")]
pub mod pure;
#[cfg(feature = "pure-tests")]
pub mod pure_test;
mod runtime;
mod terminal_input;
#[cfg(feature = "test-api")]
pub mod test_api;

pub use element_presets::{
    button, button_fit_width, static_text_display_unfocusable,
    static_text_display_unfocusable_fit_width, static_text_fit_height, static_text_fixed,
    text_input_fit_height, text_input_fixed,
};
pub use element_id::ElementId;
pub use pure::element_placement::{ElementPlacement, ParentSide};
pub use runtime::{
    ElementConfig, ElementHandler, ElementHeightMode, FocusStyle, Style, TersseUi,
    TerminalResizeHandler, TextInputBehavior, TextInputStyle, UiAsyncEngine, UiTaskQueuer,
};

pub mod prelude {
    pub use crate::{
        button, button_fit_width, static_text_display_unfocusable,
        static_text_display_unfocusable_fit_width, static_text_fit_height, static_text_fixed,
        text_input_fit_height, text_input_fixed, Color, ElementConfig, ElementHandler,
        ElementHeightMode, ElementId, ElementPlacement, FocusStyle, Location, ParentSide,
        Style, TersseUi, TerminalResizeHandler, TextInputBehavior, TextInputStyle, UiAsyncEngine,
        UiTaskQueuer, terminal_size,
    };
}

/// Standard curses terminal colors (8-color palette plus terminal default).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    /// Terminal default foreground or background (`-1` in curses).
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Location {
    pub x: u16,
    pub y: u16,
}

/// Current terminal size as `(columns, rows)`.
pub fn terminal_size() -> std::io::Result<(u16, u16)> {
    terminal_input::terminal_size()
}
