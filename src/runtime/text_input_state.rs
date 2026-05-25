use crate::pure::scroll_view;
use crate::pure::text_input::TextInputState;
use crate::pure::text_wrap;
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

    pub(super) fn apply_text_input_paste_content(&mut self, id: ElementId, paste: &str) -> bool {
        let Some(input) = self.element_by_id(id) else {
            return false;
        };
        if input.text_input.as_ref().is_none_or(|input| input.locked) {
            return false;
        }
        let state = self.text_input_state(id);
        let Some(next) = crate::pure::text_input::paste_text(&state, paste) else {
            return false;
        };
        self.set_text_input_state(id, next);
        true
    }

    pub(super) fn sync_text_input_scroll_for_cursor(&mut self, id: ElementId) {
        let Some(element) = self.element_mut_by_id(id) else {
            return;
        };
        let Some(viewport_rows) = element.fixed_viewport_height() else {
            return;
        };
        let Some(input) = element.text_input.as_ref() else {
            return;
        };
        let width = element.width.max(1);
        let total_lines = text_wrap::display_row_count(&element.text, width);
        let (line, _) = text_wrap::cursor_display_position(&element.text, input.cursor, width);
        let desired = line.saturating_sub(viewport_rows.saturating_sub(1));
        element.scroll = scroll_view::clamp_scroll_offset(desired, total_lines, viewport_rows);
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
