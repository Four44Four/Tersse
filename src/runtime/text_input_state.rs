use crate::pure::terminal_bounds;
use crate::pure::text_input::{self, TextInputState};

use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn text_input_max_visible_rows(&self, id: &str) -> Option<usize> {
        let RuntimeElement::TextInput(input) = self.element_by_id(id)? else {
            return None;
        };
        let (_, max_y) = self.win.get_max_yx();
        Some(
            terminal_bounds::rows_visible_from(input.location.y as i32, max_y).max(0) as usize,
        )
    }

    pub(super) fn text_input_state_fits_terminal(&self, id: &str, state: &TextInputState) -> bool {
        let Some(max_rows) = self.text_input_max_visible_rows(id) else {
            return false;
        };
        let Some(RuntimeElement::TextInput(input)) = self.element_by_id(id) else {
            return false;
        };
        text_input::state_fits_in_max_rows(state, input.field.width.max(1), max_rows)
    }
    pub(super) fn text_input_state(&self, id: &str) -> TextInputState {
        self.element_by_id(id)
            .and_then(RuntimeElement::text_input_state)
            .unwrap_or(TextInputState {
                text: String::new(),
                cursor: 0,
                selection_anchor: None,
            })
    }

    pub(super) fn apply_text_input_state(&mut self, id: &str, state: Option<TextInputState>) {
        if let Some(state) = state {
            if self.text_input_state_fits_terminal(id, &state) {
                self.set_text_input_state(id, state);
            }
        }
    }

    pub(super) fn apply_text_input_paste(&mut self, id: &str, state: TextInputState) {
        let Some(max_rows) = self.text_input_max_visible_rows(id) else {
            return;
        };
        let width = match self.element_by_id(id) {
            Some(RuntimeElement::TextInput(input)) => input.field.width.max(1),
            _ => return,
        };
        let clamped = text_input::clamp_state_to_max_rows(&state, width, max_rows);
        self.set_text_input_state(id, clamped);
    }

    pub(super) fn set_text_input_state(&mut self, id: &str, state: TextInputState) {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = state.text;
            input.cursor = state.cursor;
            input.selection_anchor = state.selection_anchor;
        }
    }
}
