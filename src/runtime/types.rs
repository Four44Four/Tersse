use crate::pure::text_input::TextInputState;
use crate::{
    create_button, create_text_display_element, create_text_input_field_element, Button, Color,
    Location, TextDisplayElement, TextInputField,
};

use super::RuntimeUi;

#[derive(Clone, Copy)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
}

#[derive(Clone, Copy)]
pub struct FocusStyle {
    pub focused: Style,
    pub unfocused: Style,
}

#[derive(Clone, Copy)]
pub struct TextInputStyle {
    pub focused_unlocked: Style,
    pub unfocused_unlocked: Style,
    pub focused_locked: Style,
    pub unfocused_locked: Style,
    pub selection: Style,
}

pub struct ButtonConfig {
    pub id: String,
    pub label: String,
    pub width: usize,
    pub location: Location,
    pub focus_number: f64,
    pub style: FocusStyle,
    pub on_press: ButtonHandler,
}

pub struct TextInputConfig {
    pub id: String,
    pub width: usize,
    pub location: Location,
    pub focus_number: f64,
    pub style: TextInputStyle,
    pub locked: bool,
    pub initial_text: String,
}

pub struct TextDisplayConfig {
    pub id: String,
    pub location: Location,
    pub width: usize,
    pub height: usize,
    pub focus_number: f64,
    pub style: FocusStyle,
    pub initial_text: String,
}

pub enum ElementConfig {
    Button(ButtonConfig),
    TextInput(TextInputConfig),
    TextDisplay(TextDisplayConfig),
}

pub enum UiEvent {
    None,
    Quit,
}

pub type ButtonHandler = Box<dyn FnMut(&mut RuntimeUi) + 'static>;

pub(super) struct ButtonElement {
    pub id: String,
    pub focus_number: f64,
    pub button: Button,
    pub style: FocusStyle,
    pub on_press: Option<ButtonHandler>,
}

pub(super) struct TextInputElement {
    pub id: String,
    pub focus_number: f64,
    pub location: Location,
    pub field: TextInputField,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub style: TextInputStyle,
}

pub(super) struct TextDisplayRuntimeElement {
    pub id: String,
    pub focus_number: f64,
    pub location: Location,
    pub width: usize,
    pub height: usize,
    pub scroll: usize,
    pub display: TextDisplayElement,
    pub style: FocusStyle,
}

pub(super) enum RuntimeElement {
    Button(ButtonElement),
    TextInput(TextInputElement),
    TextDisplay(TextDisplayRuntimeElement),
}

pub(crate) fn clamp_text_display_dimensions(width: usize, height: usize) -> (usize, usize) {
    (width.max(1), height.max(1))
}

pub(crate) fn text_input_state_from_parts(
    text: impl Into<String>,
    cursor: usize,
    selection_anchor: Option<usize>,
) -> TextInputState {
    TextInputState {
        text: text.into(),
        cursor,
        selection_anchor,
    }
}

impl ButtonElement {
    pub fn from_config(config: ButtonConfig) -> Self {
        let ButtonConfig {
            id,
            label,
            width,
            location,
            focus_number,
            style,
            on_press,
        } = config;

        Self {
            id,
            focus_number,
            button: create_button(
                location,
                label,
                width,
                style.unfocused.bg,
                style.unfocused.fg,
                Box::new(|| {}),
            ),
            style,
            on_press: Some(on_press),
        }
    }
}

impl TextInputElement {
    pub fn from_config(config: TextInputConfig) -> Self {
        let mut field = create_text_input_field_element(config.width);
        field.locked = config.locked;
        field.text = config.initial_text;

        Self {
            id: config.id,
            focus_number: config.focus_number,
            location: config.location,
            field,
            cursor: 0,
            selection_anchor: None,
            style: config.style,
        }
    }
}

impl TextDisplayRuntimeElement {
    pub fn from_config(config: TextDisplayConfig) -> Self {
        let (width, height) = clamp_text_display_dimensions(config.width, config.height);
        Self {
            id: config.id,
            focus_number: config.focus_number,
            location: config.location,
            width,
            height,
            scroll: 0,
            display: create_text_display_element(config.initial_text),
            style: config.style,
        }
    }
}

impl RuntimeElement {
    pub fn id(&self) -> &str {
        match self {
            RuntimeElement::Button(button) => button.id.as_str(),
            RuntimeElement::TextInput(input) => input.id.as_str(),
            RuntimeElement::TextDisplay(display) => display.id.as_str(),
        }
    }

    pub fn focus_number(&self) -> f64 {
        match self {
            RuntimeElement::Button(button) => button.focus_number,
            RuntimeElement::TextInput(input) => input.focus_number,
            RuntimeElement::TextDisplay(display) => display.focus_number,
        }
    }

    pub fn set_focus_number(&mut self, focus_number: f64) {
        match self {
            RuntimeElement::Button(button) => button.focus_number = focus_number,
            RuntimeElement::TextInput(input) => input.focus_number = focus_number,
            RuntimeElement::TextDisplay(display) => display.focus_number = focus_number,
        }
    }

    pub fn text_input_state(&self) -> Option<TextInputState> {
        let RuntimeElement::TextInput(input) = self else {
            return None;
        };
        Some(text_input_state_from_parts(
            input.field.text.clone(),
            input.cursor,
            input.selection_anchor,
        ))
    }
}
