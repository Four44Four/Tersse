use std::collections::HashMap;

use pancurses::Window;

use crate::ScreenTitle;

mod colors;
mod core;
mod elements;
mod events;
mod focus;
mod layout;
mod render;
mod text_input_state;
mod types;

pub use types::{
    ButtonConfig, ButtonHandler, ElementConfig, FocusStyle, Style, TextDisplayConfig,
    TextInputConfig, TextInputStyle, UiEvent,
};

use types::RuntimeElement;

pub struct RuntimeUi {
    win: Window,
    title: Option<ScreenTitle>,
    elements: Vec<RuntimeElement>,
    focused_position: usize,
    pair_cache: HashMap<(i16, i16), i16>,
    next_pair_id: i16,
    cached_heights: HashMap<String, usize>,
}
