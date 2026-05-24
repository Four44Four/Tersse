use crate::pure::text_input::TextInputState;
use crate::ElementId;

use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn text_input_state(&self, id: ElementId) -> TextInputState {
        self.element_by_id(id)
            .and_then(|element| element.text_input_state())
            .unwrap_or(TextInputState {
                text: String::new(),
                cursor: 0,
                selection_anchor: None,
            })
    }

    pub(super) fn apply_text_input_state(&mut self, id: ElementId, state: Option<TextInputState>) {
        if let Some(state) = state {
            self.set_text_input_state(id, state);
        }
    }

    pub(super) fn apply_text_input_paste(&mut self, id: ElementId, state: TextInputState) {
        self.set_text_input_state(id, state);
    }

    pub(super) fn set_text_input_state(&mut self, id: ElementId, state: TextInputState) {
        if let Some(element) = self.element_mut_by_id(id) {
            let Some(input) = element.text_input.as_mut() else {
                return;
            };
            element.text = state.text;
            input.cursor = state.cursor;
            input.selection_anchor = state.selection_anchor;
            self.invalidate_text_input_layout_cache(id);
        }
    }
}
