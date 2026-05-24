use crate::pure::layout_reflow;
use crate::pure::text_wrap;
use crate::ElementId;

use super::types::RuntimeElement;
use super::{RuntimeUi, TextInputLayoutCache};

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
        let dynamic_ids = self
            .elements
            .iter()
            .filter_map(|element| {
                if element.text_input.is_some() && element.fixed_viewport_height().is_none() {
                    return Some(element.id);
                }
                if element.is_fit_static_display() {
                    return Some(element.id);
                }
                None
            })
            .collect::<Vec<_>>();

        let mut relayout = false;
        for id in dynamic_ids {
            let element_id = ElementId::from_internal(id);
            let old_height = self.cached_heights.get(&id).copied().unwrap_or(1);
            let new_height = self
                .dynamic_element_render_height(element_id)
                .unwrap_or(old_height);
            let delta = layout_reflow::height_delta(old_height, new_height);
            if delta == 0 {
                continue;
            }
            if delta > 0 {
                if let Some(location) = self.element_location(element_id) {
                    let min_y = layout_reflow::min_y_after_change(location.y, old_height);
                    self.push_elements_down_from(min_y, delta, &[id]);
                }
            } else {
                relayout = true;
            }
        }
        if relayout {
            self.relayout_all_from_placements();
        }
        self.refresh_height_cache();
    }

    /// Logical row span used for reflow (content height, not viewport-clipped height).
    pub(super) fn element_render_height(&self, element: &RuntimeElement) -> usize {
        if let Some(height) = element.fixed_viewport_height() {
            if element.text_input.is_some() || element.on_activate.is_none() {
                return render_height_for_text_display(height);
            }
        }
        if element.text_input.is_some() || element.is_fit_static_display() {
            let width = element.width.max(1);
            return render_height_for_text_input_text(&element.text, width);
        }
        if element.is_button() {
            return render_height_for_button();
        }
        match element.height_mode {
            super::types::ElementHeightMode::Fixed(height) => {
                render_height_for_text_display(height)
            }
            super::types::ElementHeightMode::FitContent => render_height_for_button(),
        }
    }

    pub(super) fn dynamic_element_render_height(&mut self, id: ElementId) -> Option<usize> {
        let element = self.element_by_id(id)?;
        if element.fixed_viewport_height().is_some() {
            return None;
        }
        if element.text_input.is_some() {
            return self.text_input_render_height(id);
        }
        if element.is_fit_static_display() {
            let width = element.width.max(1);
            return Some(render_height_for_text_input_text(&element.text, width));
        }
        None
    }

    pub(super) fn text_input_render_height(&mut self, id: ElementId) -> Option<usize> {
        let element = self.element_by_id(id)?;
        if element.text_input.is_none() {
            return None;
        }
        if let Some(height) = element.fixed_viewport_height() {
            return Some(render_height_for_text_display(height));
        }
        let width = element.width.max(1);
        let text_len = element.text.len();
        if let Some(cache) = self.text_input_layout_cache.get(&id.as_internal()) {
            if cache.text_len == text_len && cache.width == width {
                return Some(cache.height);
            }
        }
        let height = render_height_for_text_input_text(&element.text, width);
        self.text_input_layout_cache.insert(
            id.as_internal(),
            TextInputLayoutCache {
                text_len,
                width,
                height,
            },
        );
        Some(height)
    }

    pub(super) fn invalidate_text_input_layout_cache(&mut self, id: ElementId) {
        self.text_input_layout_cache.remove(&id.as_internal());
    }

    pub(super) fn refresh_height_cache(&mut self) {
        self.cached_heights.clear();
        let ids: Vec<usize> = self.elements.iter().map(|e| e.id()).collect();
        for id in ids {
            let element_id = ElementId::from_internal(id);
            let height = match self.element_by_id(element_id) {
                Some(other) if other.text_input.is_some() => {
                    self.text_input_render_height(element_id).unwrap_or(1)
                }
                Some(other) if other.is_fit_static_display() => {
                    self.dynamic_element_render_height(element_id).unwrap_or(1)
                }
                Some(other) => self.element_render_height(other),
                None => 1,
            };
            self.cached_heights.insert(id, height);
        }
    }
}
