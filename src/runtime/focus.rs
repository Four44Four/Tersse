use crate::pure::focus_order;

use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn focus_next(&mut self) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        self.focused_position = focus_order::next_index(self.focused_position, order.len());
        self.sync_focus_flags();
    }

    pub(super) fn focus_prev(&mut self) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        self.focused_position = focus_order::prev_index(self.focused_position, order.len());
        self.sync_focus_flags();
    }

    fn focus_order(&self) -> Vec<String> {
        let entries = self
            .elements
            .iter()
            .map(|element| (element.focus_index(), element.id().to_string()))
            .collect::<Vec<_>>();
        focus_order::sorted_ids(entries)
    }

    pub(super) fn current_focused_id(&self) -> Option<String> {
        let order = self.focus_order();
        order.get(self.focused_position).cloned()
    }

    pub(super) fn sync_focus_position(&mut self) {
        let order = self.focus_order();
        self.focused_position = focus_order::normalize_index(self.focused_position, order.len());
        self.sync_focus_flags();
    }

    pub(super) fn sync_focus_flags(&mut self) {
        let focused = self.current_focused_id();
        for element in &mut self.elements {
            let is_focused = focused.as_deref() == Some(element.id());
            match element {
                RuntimeElement::Button(button) => button.button.focused = is_focused,
                RuntimeElement::TextInput(input) => input.field.focused = is_focused,
                RuntimeElement::TextDisplay(display) => display.display.focused = is_focused,
            }
        }
    }
}
