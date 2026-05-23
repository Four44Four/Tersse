use crate::pure::layout_reflow;
use crate::pure::text_wrap;

use super::types::RuntimeElement;
use super::RuntimeUi;

pub(crate) fn render_height_for_button() -> usize {
    1
}

pub(crate) fn render_height_for_text_input_text(text: &str, width: usize) -> usize {
    text_wrap::display_row_count(text, width.max(1))
}

pub(crate) fn render_height_for_text_display(height: usize) -> usize {
    height.max(1)
}

impl RuntimeUi {
    pub(super) fn auto_reflow_for_dynamic_heights(&mut self) {
        let text_input_ids = self
            .elements
            .iter()
            .filter_map(|element| match element {
                RuntimeElement::TextInput(input) => Some(input.id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        for id in text_input_ids {
            let old_height = self.cached_heights.get(&id).copied().unwrap_or(1);
            let new_height = self.element_render_height_by_id(&id).unwrap_or(old_height);
            let delta = layout_reflow::height_delta(old_height, new_height);
            if delta == 0 {
                continue;
            }
            if let Some(location) = self.element_location(&id) {
                let min_y = layout_reflow::min_y_after_change(location.y, old_height);
                self.shift_elements_from_min_y(&id, min_y, delta);
            }
        }
        self.refresh_height_cache();
    }

    /// Shifts every element at `y >= min_y` (except `source_id`) by `delta` rows.
    pub(super) fn shift_elements_from_min_y(&mut self, source_id: &str, min_y: u16, delta: i32) {
        if delta == 0 {
            return;
        }
        for element in &mut self.elements {
            if element.id() == source_id {
                continue;
            }

            let current_y = match element {
                RuntimeElement::Button(button) => button.button.location.y,
                RuntimeElement::TextInput(input) => input.location.y,
                RuntimeElement::TextDisplay(display) => display.location.y,
            };
            let shifted_y = layout_reflow::shifted_y(current_y, min_y, delta);
            match element {
                RuntimeElement::Button(button) => button.button.location.y = shifted_y,
                RuntimeElement::TextInput(input) => input.location.y = shifted_y,
                RuntimeElement::TextDisplay(display) => display.location.y = shifted_y,
            }
        }
    }

    pub(super) fn element_render_height_by_id(&self, id: &str) -> Option<usize> {
        self.element_by_id(id).map(Self::element_render_height)
    }

    pub(super) fn element_render_height(element: &RuntimeElement) -> usize {
        match element {
            RuntimeElement::Button(_) => render_height_for_button(),
            RuntimeElement::TextInput(input) => {
                render_height_for_text_input_text(&input.field.text, input.field.width)
            }
            RuntimeElement::TextDisplay(display) => render_height_for_text_display(display.height),
        }
    }

    pub(super) fn refresh_height_cache(&mut self) {
        self.cached_heights.clear();
        for element in &self.elements {
            self.cached_heights.insert(
                element.id().to_string(),
                Self::element_render_height(element),
            );
        }
    }
}
