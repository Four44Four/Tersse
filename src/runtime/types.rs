use crate::pure::text_input::TextInputState;
use crate::{
    create_button, create_text_display_element, create_text_input_field_element, Button, Color,
    ElementPlacement, Location, TextDisplayElement, TextInputField,
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
    pub label: String,
    pub width: usize,
    pub placement: ElementPlacement,
    pub focus_number: f64,
    pub style: FocusStyle,
    pub on_press: ButtonHandler,
}

pub struct TextInputConfig {
    pub width: usize,
    pub placement: ElementPlacement,
    pub focus_number: f64,
    pub style: TextInputStyle,
    pub locked: bool,
    pub initial_text: String,
}

pub struct TextDisplayConfig {
    pub placement: ElementPlacement,
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
    pub id: usize,
    pub focus_number: f64,
    pub placement: ElementPlacement,
    pub button: Button,
    pub style: FocusStyle,
    pub on_press: Option<ButtonHandler>,
}

pub(super) struct TextInputElement {
    pub id: usize,
    pub focus_number: f64,
    pub placement: ElementPlacement,
    pub location: Location,
    pub field: TextInputField,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub style: TextInputStyle,
}

pub(super) struct TextDisplayRuntimeElement {
    pub id: usize,
    pub focus_number: f64,
    pub placement: ElementPlacement,
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
    pub fn from_config(id: usize, config: ButtonConfig, location: Location) -> Self {
        let ButtonConfig {
            label,
            width,
            placement,
            focus_number,
            style,
            on_press,
        } = config;

        Self {
            id,
            focus_number,
            placement,
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
    pub fn from_config(id: usize, config: TextInputConfig, location: Location) -> Self {
        let mut field = create_text_input_field_element(config.width);
        field.locked = config.locked;
        field.text = config.initial_text;

        Self {
            id,
            focus_number: config.focus_number,
            placement: config.placement,
            location,
            field,
            cursor: 0,
            selection_anchor: None,
            style: config.style,
        }
    }
}

impl TextDisplayRuntimeElement {
    pub fn from_config(id: usize, config: TextDisplayConfig, location: Location) -> Self {
        let (width, height) = clamp_text_display_dimensions(config.width, config.height);
        Self {
            id,
            focus_number: config.focus_number,
            placement: config.placement,
            location,
            width,
            height,
            scroll: 0,
            display: create_text_display_element(config.initial_text),
            style: config.style,
        }
    }
}

impl RuntimeElement {
    pub fn id(&self) -> usize {
        match self {
            RuntimeElement::Button(button) => button.id,
            RuntimeElement::TextInput(input) => input.id,
            RuntimeElement::TextDisplay(display) => display.id,
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
