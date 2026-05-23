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
    pub focus_index: usize,
    pub style: FocusStyle,
    pub on_press: ButtonHandler,
}

pub struct TextInputConfig {
    pub id: String,
    pub width: usize,
    pub location: Location,
    pub focus_index: usize,
    pub style: TextInputStyle,
    pub locked: bool,
    pub initial_text: String,
}

pub struct TextDisplayConfig {
    pub id: String,
    pub location: Location,
    pub width: usize,
    pub height: usize,
    pub focus_index: usize,
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
    pub focus_index: usize,
    pub button: Button,
    pub style: FocusStyle,
    pub on_press: Option<ButtonHandler>,
}

pub(super) struct TextInputElement {
    pub id: String,
    pub focus_index: usize,
    pub location: Location,
    pub field: TextInputField,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub style: TextInputStyle,
}

pub(super) struct TextDisplayRuntimeElement {
    pub id: String,
    pub focus_index: usize,
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

impl ButtonElement {
    pub fn from_config(config: ButtonConfig) -> Self {
        let ButtonConfig {
            id,
            label,
            width,
            location,
            focus_index,
            style,
            on_press,
        } = config;

        Self {
            id,
            focus_index,
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
            focus_index: config.focus_index,
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
        Self {
            id: config.id,
            focus_index: config.focus_index,
            location: config.location,
            width: config.width.max(1),
            height: config.height.max(1),
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

    pub fn focus_index(&self) -> usize {
        match self {
            RuntimeElement::Button(button) => button.focus_index,
            RuntimeElement::TextInput(input) => input.focus_index,
            RuntimeElement::TextDisplay(display) => display.focus_index,
        }
    }

    pub fn text_input_state(&self) -> Option<TextInputState> {
        let RuntimeElement::TextInput(input) = self else {
            return None;
        };
        Some(TextInputState {
            text: input.field.text.clone(),
            cursor: input.cursor,
            selection_anchor: input.selection_anchor,
        })
    }
}
