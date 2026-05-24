//! Convenience constructors for common composed element configurations.

use crate::pure::element_placement::ElementPlacement;
use crate::runtime::{
    ElementConfig, ElementHandler, FocusStyle, TextInputBehavior, TextInputStyle,
};

/// Fixed-width, fixed-height button with a Space/Enter activate handler.
pub fn button(
    placement: ElementPlacement,
    width: usize,
    height: usize,
    focus_number: f64,
    style: FocusStyle,
    text: impl Into<String>,
    on_activate: ElementHandler,
) -> ElementConfig {
    ElementConfig::new(placement, width, focus_number, style)
        .with_fixed_height(height)
        .with_text(text)
        .with_on_activate(on_activate)
}

/// Button whose width is sized to the label text.
pub fn button_fit_width(
    placement: ElementPlacement,
    height: usize,
    focus_number: f64,
    style: FocusStyle,
    text: impl Into<String>,
    on_activate: ElementHandler,
) -> ElementConfig {
    let text = text.into();
    let width = text.chars().count().max(1);
    button(placement, width, height, focus_number, style, text, on_activate)
}

/// Static read-only text with fixed width and height; overflow is clipped and scrollable
/// with Alt/Meta + Up/Down while focused.
pub fn static_text_fixed(
    placement: ElementPlacement,
    width: usize,
    height: usize,
    focus_number: f64,
    style: FocusStyle,
    text: impl Into<String>,
) -> ElementConfig {
    ElementConfig::new(placement, width, focus_number, style)
        .with_fixed_height(height)
        .with_text(text)
}

/// Static read-only text with fixed width; height grows to fit wrapped content.
pub fn static_text_fit_height(
    placement: ElementPlacement,
    width: usize,
    focus_number: f64,
    style: FocusStyle,
    text: impl Into<String>,
) -> ElementConfig {
    ElementConfig::new(placement, width, focus_number, style)
        .with_fit_content_height()
        .with_text(text)
}

/// Text input with fixed width and height; overflow is clipped and scrollable with
/// Alt/Meta + Up/Down while focused.
pub fn text_input_fixed(
    placement: ElementPlacement,
    width: usize,
    height: usize,
    focus_number: f64,
    style: FocusStyle,
    input_style: TextInputStyle,
    text: impl Into<String>,
    locked: bool,
) -> ElementConfig {
    ElementConfig::new(placement, width, focus_number, style)
        .with_fixed_height(height)
        .with_text(text)
        .with_text_input(TextInputBehavior::new(input_style).with_locked(locked))
}

/// Text input with fixed width; height grows as wrapped content grows.
pub fn text_input_fit_height(
    placement: ElementPlacement,
    width: usize,
    focus_number: f64,
    style: FocusStyle,
    input_style: TextInputStyle,
    text: impl Into<String>,
    locked: bool,
) -> ElementConfig {
    ElementConfig::new(placement, width, focus_number, style)
        .with_fit_content_height()
        .with_text(text)
        .with_text_input(TextInputBehavior::new(input_style).with_locked(locked))
}
