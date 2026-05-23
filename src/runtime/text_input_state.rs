use crate::pure::text_input::TextInputState;

use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
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
            self.set_text_input_state(id, state);
        }
    }

    pub(super) fn set_text_input_state(&mut self, id: &str, state: TextInputState) {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = state.text;
            input.cursor = state.cursor;
            input.selection_anchor = state.selection_anchor;
        }
    }
}
