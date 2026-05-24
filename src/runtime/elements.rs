use crate::pure::element_id::allocate_element_id;
use crate::pure::layout_reflow;
use crate::ElementId;
use crate::Location;

use super::types::{
    ButtonConfig, ButtonElement, ElementConfig, RuntimeElement, TextDisplayConfig,
    TextDisplayRuntimeElement, TextInputConfig, TextInputElement,
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

    pub fn create_button(&mut self, config: ButtonConfig) -> ElementId {
        let id = self.create_element_id();
        let focused_id = self.current_focused_id();
        let (width, height) = Self::button_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::Button(ButtonElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
        self.mark_element_and_below_changed(id);
        id
    }

    pub fn update_button(&mut self, id: ElementId, config: ButtonConfig) -> bool {
        let old_bounds = self.element_bounds(id);
        if !self.elements.contains_id(id.as_internal()) {
            return false;
        }
        let focused_id = self.current_focused_id();
        let (width, height) = Self::button_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::Button(ButtonElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
        self.mark_from_y_changed(
            old_bounds
                .zip(self.element_bounds(id))
                .map(|(old, new)| old.y.min(new.y))
                .unwrap_or_default(),
        );
        true
    }

    pub fn button_width(&self, id: ElementId) -> Option<usize> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.width),
            _ => None,
        }
    }

    pub fn create_text_input(&mut self, config: TextInputConfig) -> ElementId {
        let id = self.create_element_id();
        let focused_id = self.current_focused_id();
        let (width, height) = Self::text_input_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::TextInput(TextInputElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.invalidate_text_input_layout_cache(id);
        self.refresh_height_cache();
        self.mark_element_and_below_changed(id);
        id
    }

    pub fn update_text_input(&mut self, id: ElementId, config: TextInputConfig) -> bool {
        let old_bounds = self.element_bounds(id);
        if !self.elements.contains_id(id.as_internal()) {
            return false;
        }
        let focused_id = self.current_focused_id();
        let (width, height) = Self::text_input_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::TextInput(TextInputElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.invalidate_text_input_layout_cache(id);
        self.refresh_height_cache();
        self.mark_from_y_changed(
            old_bounds
                .zip(self.element_bounds(id))
                .map(|(old, new)| old.y.min(new.y))
                .unwrap_or_default(),
        );
        true
    }

    pub fn create_text_display(&mut self, config: TextDisplayConfig) -> ElementId {
        let id = self.create_element_id();
        let focused_id = self.current_focused_id();
        let (width, height) = Self::text_display_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::TextDisplay(TextDisplayRuntimeElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
        self.mark_element_and_below_changed(id);
        id
    }

    pub fn update_text_display(&mut self, id: ElementId, config: TextDisplayConfig) -> bool {
        let old_bounds = self.element_bounds(id);
        if !self.elements.contains_id(id.as_internal()) {
            return false;
        }
        let focused_id = self.current_focused_id();
        let (width, height) = Self::text_display_config_dimensions(&config);
        let location =
            self.resolve_config_location(id.as_internal(), &config.placement, width, height);
        let element = RuntimeElement::TextDisplay(TextDisplayRuntimeElement::from_config(
            id.as_internal(),
            config,
            location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
        self.mark_from_y_changed(
            old_bounds
                .zip(self.element_bounds(id))
                .map(|(old, new)| old.y.min(new.y))
                .unwrap_or_default(),
        );
        true
    }

    pub fn create_and_reflow(&mut self, config: ElementConfig) -> ElementId {
        match config {
            ElementConfig::Button(cfg) => self.create_button(cfg),
            ElementConfig::TextInput(cfg) => self.create_text_input(cfg),
            ElementConfig::TextDisplay(cfg) => self.create_text_display(cfg),
        }
    }

    pub fn remove_and_reflow(&mut self, id: ElementId) -> bool {
        let Some(bounds) = self.element_bounds(id) else {
            return false;
        };
        let min_y = layout_reflow::min_y_after_change(bounds.y, bounds.height);
        let rows = bounds.height as u16;
        if !self.remove_element_cascade(id) {
            return false;
        }
        if rows > 0 {
            self.pull_elements_up_from(min_y, rows, &[]);
        }
        self.refresh_height_cache();
        self.mark_from_y_changed(bounds.y);
        true
    }

    pub fn remove_element(&mut self, id: ElementId) -> bool {
        let old_y = self.element_location(id).map(|location| location.y);
        let focused_id = self.current_focused_id();
        if self.elements.remove(id.as_internal()).is_some() {
            self.restore_focus(focused_id);
            self.cached_heights.remove(&id.as_internal());
            self.invalidate_text_input_layout_cache(id);
            if let Some(y) = old_y {
                self.mark_from_y_changed(y);
            }
            true
        } else {
            false
        }
    }

    pub fn element_location(&self, id: ElementId) -> Option<Location> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.location),
            Some(RuntimeElement::TextInput(input)) => Some(input.location),
            Some(RuntimeElement::TextDisplay(display)) => Some(display.location),
            None => None,
        }
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
        self.shift_element_subtree(id, delta_x as i16, delta_y);
        self.recompute_all_relative_locations();
        self.mark_from_y_changed(old_location.y.min(location.y));
        true
    }

    pub fn set_text_display_dimensions(
        &mut self,
        id: ElementId,
        width: usize,
        height: usize,
    ) -> bool {
        let anchor_y = self.element_location(id).map(|location| location.y).unwrap_or_default();
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.width = width.max(1);
            display.height = height.max(1);
            self.recompute_all_relative_locations();
            self.mark_from_y_changed(anchor_y);
            true
        } else {
            false
        }
    }

    pub fn set_text_display_text(&mut self, id: ElementId, text: impl Into<String>) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.display.text = text.into();
            display.scroll = 0;
            self.mark_element_only_changed(id);
            true
        } else {
            false
        }
    }

    pub fn read_text_input(&self, id: ElementId) -> Option<String> {
        match self.element_by_id(id) {
            Some(RuntimeElement::TextInput(input)) => Some(input.field.text.clone()),
            _ => None,
        }
    }

    pub fn set_text_input_text(&mut self, id: ElementId, text: impl Into<String>) -> bool {
        let Some(old_height) = self.text_input_render_height(id) else {
            return false;
        };
        let anchor_y = self.element_location(id).map(|location| location.y).unwrap_or_default();
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = text.into();
            input.cursor = input.field.text.chars().count();
            input.selection_anchor = None;
            self.invalidate_text_input_layout_cache(id);
            self.recompute_all_relative_locations();
            let new_height = self.text_input_render_height(id).unwrap_or(old_height);
            if old_height != new_height {
                self.mark_from_y_changed(anchor_y);
            } else {
                self.mark_element_only_changed(id);
            }
            true
        } else {
            false
        }
    }

    pub fn set_text_input_lock_status(&mut self, id: ElementId, locked: bool) -> bool {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.locked = locked;
            if locked {
                input.selection_anchor = None;
            }
            self.mark_element_only_changed(id);
            true
        } else {
            false
        }
    }

    pub(super) fn element_by_id(&self, id: ElementId) -> Option<&RuntimeElement> {
        self.elements.get(id.as_internal())
    }

    pub(super) fn element_mut_by_id(&mut self, id: ElementId) -> Option<&mut RuntimeElement> {
        self.elements.get_mut(id.as_internal())
    }
}
