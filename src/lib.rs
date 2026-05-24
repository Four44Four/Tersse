//! Public API for building and styling TUI elements.

pub mod clipboard;
mod constants;
pub mod element_presets;
mod element_store;
#[cfg(feature = "pure-tests")]
pub mod pure;
#[cfg(not(feature = "pure-tests"))]
mod pure;
mod runtime;
mod terminal_input;

pub use constants::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;
pub use element_presets::{
    button, button_fit_width, static_text_fit_height, static_text_fixed, text_input_fit_height,
    text_input_fixed,
};
pub use element_store::{ElementId, ElementStore, StoredElement};
pub use pure::element_placement::{ElementBounds, ElementPlacement, ParentSide};
pub use runtime::{
    runtime_clamp_fixed_height, runtime_render_height_for_element_text,
    runtime_terminal_color_code, runtime_text_input_state_snapshot, ElementConfig, ElementHandler,
    ElementHeightMode, FocusStyle, RuntimeUi, Style, TextInputBehavior, TextInputStyle, UiRuntime,
    UiSession,
};

pub mod prelude {
    pub use crate::{
        button, button_fit_width, static_text_fit_height, static_text_fixed, text_input_fit_height,
        text_input_fixed, Color, ElementConfig, ElementHandler, ElementHeightMode, ElementId,
        ElementPlacement, FocusStyle, Location, ParentSide, RuntimeUi, Style, TextInputBehavior,
        TextInputStyle, UiRuntime, UiSession,
    };
}

use std::error::Error;
use std::fmt::{Display, Formatter};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputProperty {
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub width: usize,
    pub text: String,
    pub focused: bool,
    pub text_input: Option<TextInputProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenTitle {
    pub text: String,
    pub alignment: TitleAlignment,
    pub fg_color: Color,
    pub bg_color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleAlignment {
    Left,
    Right,
    Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FocusError {
    IdNotFound { id: ElementId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeleteElementError {
    IdNotFound { id: ElementId },
    NoFocusedElement,
}

impl Display for FocusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FocusError::IdNotFound { id } => {
                write!(
                    f,
                    "cannot focus element {id:?}; no element with that id exists"
                )
            }
        }
    }
}

impl Error for FocusError {}

impl Display for DeleteElementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteElementError::IdNotFound { id } => {
                write!(
                    f,
                    "cannot delete element {id:?}; no element with that id exists"
                )
            }
            DeleteElementError::NoFocusedElement => {
                write!(f, "cannot delete focused element; no element is focused")
            }
        }
    }
}

impl Error for DeleteElementError {}

/// Creates a plain text element with fixed width and no input behavior.
pub fn create_text_element(width: usize, initial_text: impl Into<String>) -> Element {
    Element {
        width,
        text: initial_text.into(),
        focused: false,
        text_input: None,
    }
}

/// Enables or disables text-input behavior for an element.
pub fn set_element_text_input_property(element: &mut Element, property: Option<TextInputProperty>) {
    element.text_input = property;
}

/// Sets lock status of the element text-input behavior.
pub fn set_element_lock_status(element: &mut Element, locked: bool) -> bool {
    let Some(input) = element.text_input.as_mut() else {
        return false;
    };
    input.locked = locked;
    true
}

/// Reads text from an element and returns it as a string.
pub fn read_text_from_element(element: &Element) -> String {
    element.text.clone()
}

/// Updates text content of an element.
pub fn update_text_of_element(element: &mut Element, updated_text: impl Into<String>) {
    element.text = updated_text.into();
}

/// Forces focus onto exactly one element by id.
pub fn force_focus_on_element(store: &mut ElementStore, id: ElementId) -> Result<(), FocusError> {
    if store.get(id).is_none() {
        return Err(FocusError::IdNotFound { id });
    }
    for stored in store.iter_mut() {
        let is_focused = stored.id() == id;
        stored.element.focused = is_focused;
    }
    Ok(())
}

/// Deletes a TUI element by id and returns the removed entry.
pub fn delete_tui_element(
    store: &mut ElementStore,
    id: ElementId,
) -> Result<StoredElement, DeleteElementError> {
    store
        .remove(id)
        .ok_or(DeleteElementError::IdNotFound { id })
}

/// Deletes the currently focused TUI element and returns it.
pub fn delete_focused_tui_element(
    store: &mut ElementStore,
) -> Result<StoredElement, DeleteElementError> {
    let focused_id = store
        .iter()
        .find(|stored| stored.element.focused)
        .map(|stored| stored.id());
    let Some(id) = focused_id else {
        return Err(DeleteElementError::NoFocusedElement);
    };
    store.remove(id).ok_or(DeleteElementError::NoFocusedElement)
}

/// Sets title of the current screen.
pub fn set_title_of_current_screen(
    text_string: impl Into<String>,
    alignment: TitleAlignment,
    fg_color: Color,
    bg_color: Color,
) -> ScreenTitle {
    ScreenTitle {
        text: text_string.into(),
        alignment,
        fg_color,
        bg_color,
    }
}
