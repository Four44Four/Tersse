//! Test-only API surface. Not part of the public crate API for library users.
//!
//! Enable with the `test-api` Cargo feature (also enables `pure-tests` for `tersse::pure`).

use std::error::Error;
use std::fmt::{Display, Formatter};

pub use crate::constants::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;
pub use crate::element_id::ElementId;
pub use crate::legacy_element_store::{
    DeleteElementError, Element, ElementStore, FocusError, StoredElement, TextInputProperty,
};
pub use crate::pure::element_placement::ElementBounds;
pub use crate::runtime::{
    runtime_clamp_fixed_height, runtime_render_height_for_element_text,
    runtime_terminal_color_code, runtime_text_input_state_snapshot,
};

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
