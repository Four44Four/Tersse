use crate::pure::text_input::TextInputState;
use crate::{Color, ElementPlacement, Location};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementHeightMode {
    Fixed(usize),
    FitContent,
}

pub struct TextInputBehavior {
    pub locked: bool,
    pub style: TextInputStyle,
}

pub struct ElementConfig {
    pub placement: ElementPlacement,
    pub width: usize,
    pub height_mode: ElementHeightMode,
    pub focus_number: f64,
    pub text: String,
    pub style: FocusStyle,
    pub on_activate: Option<ElementHandler>,
    pub text_input: Option<TextInputBehavior>,
}

pub(super) enum UiEvent {
    None,
    Quit,
}

pub type ElementHandler = Box<dyn FnMut(&mut RuntimeUi) + 'static>;

impl ElementConfig {
    pub fn new(
        placement: ElementPlacement,
        width: usize,
        focus_number: f64,
        style: FocusStyle,
    ) -> Self {
        Self {
            placement,
            width: width.max(1),
            height_mode: ElementHeightMode::FitContent,
            focus_number,
            text: String::new(),
            style,
            on_activate: None,
            text_input: None,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width.max(1);
        self
    }

    pub fn with_fixed_height(mut self, height: usize) -> Self {
        self.height_mode = ElementHeightMode::Fixed(height.max(1));
        self
    }

    pub fn with_fit_content_height(mut self) -> Self {
        self.height_mode = ElementHeightMode::FitContent;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_on_activate(mut self, on_activate: ElementHandler) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn with_text_input(mut self, text_input: TextInputBehavior) -> Self {
        self.text_input = Some(text_input);
        self
    }
}

impl TextInputBehavior {
    pub fn new(style: TextInputStyle) -> Self {
        Self {
            locked: false,
            style,
        }
    }

    pub fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }
}

pub(super) struct RuntimeTextInput {
    pub locked: bool,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub style: TextInputStyle,
}

pub(super) struct RuntimeElement {
    pub id: usize,
    pub focus_number: f64,
    pub placement: ElementPlacement,
    pub location: Location,
    pub width: usize,
    pub height_mode: ElementHeightMode,
    pub scroll: usize,
    pub text: String,
    pub focused: bool,
    pub style: FocusStyle,
    pub on_activate: Option<ElementHandler>,
    pub text_input: Option<RuntimeTextInput>,
}

pub(crate) fn clamp_fixed_height(height: usize) -> usize {
    height.max(1)
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

impl RuntimeElement {
    pub fn is_button(&self) -> bool {
        self.on_activate.is_some() && self.text_input.is_none()
    }

    pub fn is_fit_static_display(&self) -> bool {
        self.text_input.is_none()
            && self.on_activate.is_none()
            && matches!(self.height_mode, ElementHeightMode::FitContent)
    }

    pub fn fixed_viewport_height(&self) -> Option<usize> {
        match self.height_mode {
            ElementHeightMode::Fixed(height) => Some(height.max(1)),
            ElementHeightMode::FitContent => None,
        }
    }

    pub fn from_config(id: usize, config: ElementConfig, location: Location) -> Self {
        Self {
            id,
            focus_number: config.focus_number,
            placement: config.placement,
            location,
            width: config.width.max(1),
            height_mode: config.height_mode,
            scroll: 0,
            text: config.text,
            focused: false,
            style: config.style,
            on_activate: config.on_activate,
            text_input: config.text_input.map(|behavior| RuntimeTextInput {
                locked: behavior.locked,
                cursor: 0,
                selection_anchor: None,
                style: behavior.style,
            }),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn focus_number(&self) -> f64 {
        self.focus_number
    }

    pub fn set_focus_number(&mut self, focus_number: f64) {
        self.focus_number = focus_number;
    }

    pub fn text_input_state(&self) -> Option<TextInputState> {
        let input = self.text_input.as_ref()?;
        Some(text_input_state_from_parts(
            self.text.clone(),
            input.cursor,
            input.selection_anchor,
        ))
    }
}
