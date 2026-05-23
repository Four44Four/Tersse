use crate::pure::focus_order;
use crate::ElementId;

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

    fn focus_order(&self) -> Vec<usize> {
        self.elements.focus_order_ids()
    }

    pub(super) fn current_focused_id(&self) -> Option<ElementId> {
        let order = self.focus_order();
        order
            .get(self.focused_position)
            .copied()
            .map(ElementId::from_internal)
    }

    pub(super) fn restore_focus(&mut self, focused_id: Option<ElementId>) {
        let order = self.focus_order();
        let focused_internal = focused_id.map(|id| id.as_internal());
        self.focused_position = focus_order::index_for_focused_id(
            &order,
            focused_internal,
            self.focused_position,
        );
        self.sync_focus_flags();
    }

    pub(super) fn sync_focus_flags(&mut self) {
        let focused = self.current_focused_id();
        for element in self.elements.iter_mut() {
            let is_focused =
                focused == Some(ElementId::from_internal(element.id()));
            match element {
                RuntimeElement::Button(button) => button.button.focused = is_focused,
                RuntimeElement::TextInput(input) => input.field.focused = is_focused,
                RuntimeElement::TextDisplay(display) => display.display.focused = is_focused,
            }
        }
    }
}
