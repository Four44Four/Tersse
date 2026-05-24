//! Public API for building and styling TUI elements.

pub mod clipboard;
mod constants;
mod element_store;
#[cfg(feature = "pure-tests")]
pub mod pure;
#[cfg(not(feature = "pure-tests"))]
mod pure;
mod runtime;
mod terminal_input;

pub use element_store::{ElementId, ElementStore, StoredElement};
pub use pure::element_placement::{ElementBounds, ElementPlacement, ParentSide};
pub use runtime::{
    ButtonConfig, ButtonHandler, ElementConfig, FocusStyle, RuntimeUi, Style, TextDisplayConfig,
    TextInputConfig, TextInputStyle, UiSession, runtime_clamp_text_display_dimensions,
    runtime_render_height_for_button, runtime_render_height_for_text_display,
    runtime_render_height_for_text_input_text, runtime_terminal_color_code,
    runtime_text_input_state_snapshot,
};
pub use constants::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;

pub mod prelude {
    pub use crate::{
        ButtonConfig, Color, ElementId, ElementPlacement, FocusStyle, Location, ParentSide,
        RuntimeUi, Style, TextDisplayConfig, TextInputConfig, TextInputStyle, UiSession,
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

pub type ButtonCallback = Box<dyn FnMut() + Send + 'static>;

pub struct Button {
    pub location: Location,
    pub display_string: String,
    pub width: usize,
    pub bg_color: Color,
    pub fg_color: Color,
    pub focused: bool,
    pub callback_action: ButtonCallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputField {
    pub width: usize,
    pub text: String,
    pub locked: bool,
    pub focused: bool,
    pub bg_color: Color,
    pub fg_color: Color,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDisplayElement {
    pub text: String,
    pub focused: bool,
    pub bg_color: Color,
    pub fg_color: Color,
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

pub enum Element {
    Button(Button),
    TextInputField(TextInputField),
    TextDisplayElement(TextDisplayElement),
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
                write!(f, "cannot focus element {id:?}; no element with that id exists")
            }
        }
    }
}

impl Error for FocusError {}

impl Display for DeleteElementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteElementError::IdNotFound { id } => {
                write!(f, "cannot delete element {id:?}; no element with that id exists")
            }
            DeleteElementError::NoFocusedElement => {
                write!(f, "cannot delete focused element; no element is focused")
            }
        }
    }
}

impl Error for DeleteElementError {}

/// Creates a button with location, display string, fixed width, colors, and callback.
///
/// The label is truncated to `width`. Shorter labels are padded with spaces at draw time.
pub fn create_button(
    location: Location,
    display_string: impl Into<String>,
    width: usize,
    bg_color: Color,
    fg_color: Color,
    callback_action: ButtonCallback,
) -> Button {
    let width = width.max(1);
    let display_string = crate::pure::button::truncate_label(&display_string.into(), width);
    Button {
        location,
        display_string,
        width,
        bg_color,
        fg_color,
        focused: false,
        callback_action,
    }
}

/// Creates a text input field element with the given width.
pub fn create_text_input_field_element(width: usize) -> TextInputField {
    TextInputField {
        width,
        text: String::new(),
        locked: false,
        focused: false,
        bg_color: default_non_focused_non_locked_bg_color(),
        fg_color: default_non_focused_non_locked_fg_color(),
    }
}

/// Sets lock status of a text input field.
pub fn set_text_input_field_lock_status(field: &mut TextInputField, locked: bool) {
    field.locked = locked;
}

/// Reads text from a text input field and returns it as a string.
pub fn read_text_from_text_input_field(field: &TextInputField) -> String {
    field.text.clone()
}

/// Creates a text display element.
pub fn create_text_display_element(initial_text: impl Into<String>) -> TextDisplayElement {
    TextDisplayElement {
        text: initial_text.into(),
        focused: false,
        bg_color: default_non_focused_locked_bg_color(),
        fg_color: default_non_focused_locked_fg_color(),
    }
}

/// Forces focus onto exactly one element by id.
pub fn force_focus_on_element(store: &mut ElementStore, id: ElementId) -> Result<(), FocusError> {
    if store.get(id).is_none() {
        return Err(FocusError::IdNotFound { id });
    }
    for stored in store.iter_mut() {
        let is_focused = stored.id() == id;
        set_element_focus(&mut stored.element, is_focused);
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
        .find(|stored| element_is_focused(&stored.element))
        .map(|stored| stored.id());
    let Some(id) = focused_id else {
        return Err(DeleteElementError::NoFocusedElement);
    };
    store.remove(id).ok_or(DeleteElementError::NoFocusedElement)
}

/// Changes background color of non-focused, non-locked text input field elements.
pub fn change_bg_color_of_non_focused_non_locked_text_input_field_elements(
    store: &mut ElementStore,
    bg_color: Color,
) {
    apply_color_to_matching_elements(store, bg_color, Channel::Background, |field| {
        !field.focused && !field.locked
    });
}

/// Changes foreground color of non-focused, non-locked text input field elements.
pub fn change_fg_color_of_non_focused_non_locked_text_input_field_elements(
    store: &mut ElementStore,
    fg_color: Color,
) {
    apply_color_to_matching_elements(store, fg_color, Channel::Foreground, |field| {
        !field.focused && !field.locked
    });
}

/// Changes background color of non-focused locked text input fields and text display elements.
pub fn change_bg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements(
    store: &mut ElementStore,
    bg_color: Color,
) {
    apply_color_to_locked_like_elements(store, bg_color, Channel::Background, false);
}

/// Changes foreground color of non-focused locked text input fields and text display elements.
pub fn change_fg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements(
    store: &mut ElementStore,
    fg_color: Color,
) {
    apply_color_to_locked_like_elements(store, fg_color, Channel::Foreground, false);
}

/// Changes background color of focused, non-locked text input field elements.
pub fn change_bg_color_of_focused_non_locked_text_input_field_elements(
    store: &mut ElementStore,
    bg_color: Color,
) {
    apply_color_to_matching_elements(store, bg_color, Channel::Background, |field| {
        field.focused && !field.locked
    });
}

/// Changes foreground color of focused, non-locked text input field elements.
pub fn change_fg_color_of_focused_non_locked_text_input_field_elements(
    store: &mut ElementStore,
    fg_color: Color,
) {
    apply_color_to_matching_elements(store, fg_color, Channel::Foreground, |field| {
        field.focused && !field.locked
    });
}

/// Changes background color of focused locked text input fields and text display elements.
pub fn change_bg_color_of_focused_locked_text_input_field_elements_and_text_display_elements(
    store: &mut ElementStore,
    bg_color: Color,
) {
    apply_color_to_locked_like_elements(store, bg_color, Channel::Background, true);
}

/// Changes foreground color of focused locked text input fields and text display elements.
pub fn change_fg_color_of_focused_locked_text_input_field_elements_and_text_display_elements(
    store: &mut ElementStore,
    fg_color: Color,
) {
    apply_color_to_locked_like_elements(store, fg_color, Channel::Foreground, true);
}

/// Updates text content of a text display element.
pub fn update_text_of_text_display_element(
    display_element: &mut TextDisplayElement,
    updated_text: impl Into<String>,
) {
    display_element.text = updated_text.into();
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

fn default_non_focused_non_locked_bg_color() -> Color {
    Color::Black
}

fn default_non_focused_non_locked_fg_color() -> Color {
    Color::White
}

fn default_non_focused_locked_bg_color() -> Color {
    Color::Default
}

fn default_non_focused_locked_fg_color() -> Color {
    Color::Yellow
}

fn set_element_focus(element: &mut Element, focused: bool) {
    match element {
        Element::Button(button) => button.focused = focused,
        Element::TextInputField(field) => field.focused = focused,
        Element::TextDisplayElement(display) => display.focused = focused,
    }
}

fn element_is_focused(element: &Element) -> bool {
    match element {
        Element::Button(button) => button.focused,
        Element::TextInputField(field) => field.focused,
        Element::TextDisplayElement(display) => display.focused,
    }
}

#[derive(Clone, Copy)]
enum Channel {
    Background,
    Foreground,
}

fn apply_color_to_matching_elements<F>(
    store: &mut ElementStore,
    color: Color,
    channel: Channel,
    mut predicate: F,
) where
    F: FnMut(&TextInputField) -> bool,
{
    for stored in store.iter_mut() {
        if let Element::TextInputField(field) = &mut stored.element {
            if predicate(field) {
                set_color(channel, &mut field.bg_color, &mut field.fg_color, color);
            }
        }
    }
}

fn apply_color_to_locked_like_elements(
    store: &mut ElementStore,
    color: Color,
    channel: Channel,
    focused: bool,
) {
    for stored in store.iter_mut() {
        match &mut stored.element {
            Element::TextInputField(field) if field.locked && field.focused == focused => {
                set_color(channel, &mut field.bg_color, &mut field.fg_color, color);
            }
            Element::TextDisplayElement(display) if display.focused == focused => {
                set_color(channel, &mut display.bg_color, &mut display.fg_color, color);
            }
            _ => {}
        }
    }
}

fn set_color(channel: Channel, bg_color: &mut Color, fg_color: &mut Color, color: Color) {
    match channel {
        Channel::Background => *bg_color = color,
        Channel::Foreground => *fg_color = color,
    }
}
