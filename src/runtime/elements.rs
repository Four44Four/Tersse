use crate::pure::element_id::allocate_element_id;
use crate::pure::layout_reflow;
use crate::ElementId;
use crate::Location;

use super::types::{
    ElementConfig, ElementHandler, ElementHeightMode, RuntimeElement, TextInputBehavior,
};
use super::RuntimeUi;

impl RuntimeUi {
    fn allocate_element_id(&mut self) -> usize {
        loop {
            let id = allocate_element_id(&mut self.next_element_id);
            if !self.elements.contains_id(id) {
                return id;
            }
        }
    }

    fn create_element_id(&mut self) -> ElementId {
        ElementId::from_internal(self.allocate_element_id())
    }

    pub fn create_element(&mut self, config: ElementConfig) -> ElementId {
        let before = self.capture_layout_snapshot();
        let id = self.create_element_id();
        let focused_id = self.current_focused_id();
        let (width, height) = Self::element_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::from_config(id.as_internal(), config, location);
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.invalidate_text_input_layout_cache(id);
        self.refresh_height_cache();
        self.mark_layout_redraw_after(id, before);
        id
    }

    pub fn update_element(&mut self, id: ElementId, config: ElementConfig) -> bool {
        if !self.elements.contains_id(id.as_internal()) {
            return false;
        }
        let before = self.capture_layout_snapshot();
        let focused_id = self.current_focused_id();
        let (width, height) = Self::element_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::from_config(id.as_internal(), config, location);
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.invalidate_text_input_layout_cache(id);
        self.refresh_height_cache();
        self.mark_layout_redraw_after(id, before);
        true
    }

    pub fn create_and_reflow(&mut self, config: ElementConfig) -> ElementId {
        self.create_element(config)
    }

    pub fn remove_and_reflow(&mut self, id: ElementId) -> bool {
        let Some(bounds) = self.element_bounds(id) else {
            return false;
        };
        let before = self.capture_layout_snapshot();
        let min_y = layout_reflow::min_y_after_change(bounds.y, bounds.height);
        let rows = bounds.height as u16;
        if !self.remove_element_cascade(id) {
            return false;
        }
        if rows > 0 {
            self.pull_elements_up_from(min_y, rows, &[]);
        }
        self.refresh_height_cache();
        self.mark_layout_redraw_after(id, before);
        true
    }

    pub fn remove_element(&mut self, id: ElementId) -> bool {
        let before = self.capture_layout_snapshot();
        let removed_bounds = self.element_bounds(id);
        let focused_id = self.current_focused_id();
        if self.elements.remove(id.as_internal()).is_some() {
            self.restore_focus(focused_id);
            self.cached_heights.remove(&id.as_internal());
            self.invalidate_text_input_layout_cache(id);
            if let Some(bounds) = removed_bounds {
                self.clear_element_occupied_space(bounds);
            }
            self.mark_layout_redraw_after(id, before);
            true
        } else {
            false
        }
    }

    pub fn element_location(&self, id: ElementId) -> Option<Location> {
        self.element_by_id(id).map(|element| element.location)
    }

    pub fn set_focus_number(&mut self, id: ElementId, focus_number: f64) -> bool {
        let focused_id = self.current_focused_id();
        if self
            .elements
            .set_focus_number(id.as_internal(), focus_number)
        {
            self.restore_focus(focused_id);
            true
        } else {
            false
        }
    }

    pub fn set_element_location(&mut self, id: ElementId, location: Location) -> bool {
        let Some(old_location) = self.element_location(id) else {
            return false;
        };
        let delta_x = location.x as i32 - old_location.x as i32;
        let delta_y = location.y as i32 - old_location.y as i32;
        if delta_x == 0 && delta_y == 0 {
            return true;
        }
        let before = self.capture_layout_snapshot();
        self.shift_element_subtree(id, delta_x as i16, delta_y);
        self.recompute_all_relative_locations();
        self.mark_layout_redraw_after(id, before);
        true
    }

    pub fn set_element_dimensions(
        &mut self,
        id: ElementId,
        width: usize,
        height_mode: ElementHeightMode,
    ) -> bool {
        let before = self.capture_layout_snapshot();
        if let Some(element) = self.element_mut_by_id(id) {
            element.width = width.max(1);
            element.height_mode = match height_mode {
                ElementHeightMode::Fixed(height) => ElementHeightMode::Fixed(height.max(1)),
                ElementHeightMode::FitContent => ElementHeightMode::FitContent,
            };
            self.recompute_all_relative_locations();
            self.mark_layout_redraw_after(id, before);
            true
        } else {
            false
        }
    }

    pub fn read_element_text(&self, id: ElementId) -> Option<String> {
        self.element_by_id(id).map(|element| element.text.clone())
    }

    pub fn set_element_text(&mut self, id: ElementId, text: impl Into<String>) -> bool {
        let before = self.capture_layout_snapshot();
        if let Some(element) = self.element_mut_by_id(id) {
            element.text = text.into();
            if let Some(input) = element.text_input.as_mut() {
                input.cursor = element.text.chars().count();
                input.selection_anchor = None;
            }
            element.scroll = 0;
            self.invalidate_text_input_layout_cache(id);
            self.recompute_all_relative_locations();
            self.refresh_height_cache();
            self.mark_layout_redraw_after(id, before);
            true
        } else {
            false
        }
    }

    pub fn set_element_text_input_behavior(
        &mut self,
        id: ElementId,
        behavior: Option<TextInputBehavior>,
    ) -> bool {
        let before = self.capture_layout_snapshot();
        if let Some(element) = self.element_mut_by_id(id) {
            element.text_input = behavior.map(|next| super::types::RuntimeTextInput {
                locked: next.locked,
                cursor: element.text.chars().count(),
                selection_anchor: None,
                style: next.style,
            });
            self.invalidate_text_input_layout_cache(id);
            self.recompute_all_relative_locations();
            self.refresh_height_cache();
            self.mark_layout_redraw_after(id, before);
            true
        } else {
            false
        }
    }

    pub fn set_element_lock_status(&mut self, id: ElementId, locked: bool) -> bool {
        if let Some(element) = self.element_mut_by_id(id) {
            let Some(text_input) = element.text_input.as_mut() else {
                return false;
            };
            text_input.locked = locked;
            if locked {
                text_input.selection_anchor = None;
            }
            self.mark_element_only_changed(id);
            true
        } else {
            false
        }
    }

    pub fn set_element_on_activate(
        &mut self,
        id: ElementId,
        on_activate: Option<ElementHandler>,
    ) -> bool {
        if let Some(element) = self.element_mut_by_id(id) {
            element.on_activate = on_activate;
            true
        } else {
            false
        }
    }

    pub fn element_width(&self, id: ElementId) -> Option<usize> {
        self.element_by_id(id).map(|element| element.width)
    }

    pub fn element_has_text_input(&self, id: ElementId) -> bool {
        self.element_by_id(id)
            .is_some_and(|element| element.text_input.is_some())
    }

    pub(super) fn element_by_id(&self, id: ElementId) -> Option<&RuntimeElement> {
        self.elements.get(id.as_internal())
    }

    pub(super) fn element_mut_by_id(&mut self, id: ElementId) -> Option<&mut RuntimeElement> {
        self.elements.get_mut(id.as_internal())
    }
}
