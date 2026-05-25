use crate::pure::screen_scroll;
use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input::TextInputState;
use crate::pure::text_wrap;
use crate::ElementId;

use super::RuntimeUi;

impl RuntimeUi {
    /// Terminal rows currently visible for a fit-height text field (0 when fully off-screen).
    pub(super) fn text_input_terminal_visible_rows(&self, anchor_y: u16, logical_rows: usize) -> usize {
        let (max_y, _) = self.win.get_max_yx();
        let y = self.scrolled_y(anchor_y as i32);
        terminal_bounds::clip_height_at_terminal(y, logical_rows as i32, max_y).max(0) as usize
    }
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
        let (anchor_y, fixed_viewport, total_lines, line) = {
            let Some(element) = self.element_by_id(id) else {
                return;
            };
            let Some(input) = element.text_input.as_ref() else {
                return;
            };
            let width = element.width.max(1);
            let total_lines = text_wrap::display_row_count(&element.text, width);
            let (line, _) = text_wrap::cursor_display_position(&element.text, input.cursor, width);
            (
                element.location.y,
                element.fixed_viewport_height(),
                total_lines,
                line,
            )
        };
        let viewport_rows = fixed_viewport.unwrap_or_else(|| {
            self.text_input_terminal_visible_rows(anchor_y, total_lines)
                .max(1)
        });
        let desired = line.saturating_sub(viewport_rows.saturating_sub(1));
        let scroll =
            scroll_view::clamp_scroll_offset(desired, total_lines, viewport_rows.max(1));
        if let Some(element) = self.element_mut_by_id(id) {
            element.scroll = scroll;
        }
    }

    /// Scrolls the document so the bottom of a growing fit-height text field stays reachable.
    pub(super) fn sync_screen_scroll_for_text_input_growth(&mut self, id: ElementId) {
        let Some(element) = self.element_by_id(id) else {
            return;
        };
        if element.fixed_viewport_height().is_some() || element.text_input.is_none() {
            return;
        }
        let width = element.width.max(1);
        let line_count = text_wrap::display_row_count(&element.text, width);
        let bottom_row = element.location.y as usize + line_count.saturating_sub(1);
        let (_, viewport) = self.screen_scroll_bounds();
        let (content_height, _) = self.full_screen_scroll_bounds();
        self.screen_scroll = screen_scroll::screen_scroll_to_show_row(
            bottom_row,
            content_height,
            viewport,
        );
    }

    pub(super) fn sync_text_input_viewport_after_edit(&mut self, id: ElementId) -> bool {
        self.auto_reflow_for_dynamic_heights();
        // Screen scroll first so in-field scroll uses the correct on-screen row count.
        self.sync_screen_scroll_for_text_input_growth(id);
        self.sync_text_input_scroll_for_cursor(id);
        self.element_by_id(id)
            .is_some_and(|element| element.fixed_viewport_height().is_none())
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
