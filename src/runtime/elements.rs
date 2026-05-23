use crate::pure::layout_reflow;
use crate::Location;

use super::types::{
    ButtonConfig, ButtonElement, ElementConfig, RuntimeElement, TextDisplayConfig,
    TextDisplayRuntimeElement, TextInputConfig, TextInputElement,
};
use super::RuntimeUi;

impl RuntimeUi {
    pub fn upsert_button(&mut self, config: ButtonConfig) {
        let focused_id = self.current_focused_id();
        let (width, height) = Self::button_config_dimensions(&config);
        let location = self.resolve_config_location(&config.id, &config.placement, width, height);
        let element = RuntimeElement::Button(ButtonElement::from_config(config, location));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
    }

    pub fn button_width(&self, id: &str) -> Option<usize> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.width),
            _ => None,
        }
    }

    pub fn upsert_text_input(&mut self, config: TextInputConfig) {
        let focused_id = self.current_focused_id();
        let id = config.id.clone();
        let (width, height) = Self::text_input_config_dimensions(&config);
        let location = self.resolve_config_location(&id, &config.placement, width, height);
        let element = RuntimeElement::TextInput(TextInputElement::from_config(config, location));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.invalidate_text_input_layout_cache(&id);
        self.refresh_height_cache();
    }

    pub fn upsert_text_display(&mut self, config: TextDisplayConfig) {
        let focused_id = self.current_focused_id();
        let (width, height) = Self::text_display_config_dimensions(&config);
        let location = self.resolve_config_location(&config.id, &config.placement, width, height);
        let element = RuntimeElement::TextDisplay(TextDisplayRuntimeElement::from_config(
            config, location,
        ));
        self.elements.upsert(element);
        self.recompute_all_relative_locations();
        self.restore_focus(focused_id);
        self.refresh_height_cache();
    }

    pub fn upsert_and_reflow(&mut self, config: ElementConfig) {
        match config {
            ElementConfig::Button(cfg) => self.upsert_button(cfg),
            ElementConfig::TextInput(cfg) => self.upsert_text_input(cfg),
            ElementConfig::TextDisplay(cfg) => self.upsert_text_display(cfg),
        }
    }

    pub fn remove_and_reflow(&mut self, id: &str) -> bool {
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
        true
    }

    pub fn remove_element(&mut self, id: &str) -> bool {
        let focused_id = self.current_focused_id();
        if self.elements.remove(id).is_some() {
            self.restore_focus(focused_id);
            self.cached_heights.remove(id);
            self.invalidate_text_input_layout_cache(id);
            true
        } else {
            false
        }
    }

    pub fn element_location(&self, id: &str) -> Option<Location> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.location),
            Some(RuntimeElement::TextInput(input)) => Some(input.location),
            Some(RuntimeElement::TextDisplay(display)) => Some(display.location),
            None => None,
        }
    }

    pub fn set_focus_number(&mut self, id: &str, focus_number: f64) -> bool {
        let focused_id = self.current_focused_id();
        if self.elements.set_focus_number(id, focus_number) {
            self.restore_focus(focused_id);
            true
        } else {
            false
        }
    }

    pub fn set_element_location(&mut self, id: &str, location: Location) -> bool {
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
        true
    }

    pub fn set_text_display_dimensions(&mut self, id: &str, width: usize, height: usize) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.width = width.max(1);
            display.height = height.max(1);
            self.recompute_all_relative_locations();
            true
        } else {
            false
        }
    }

    pub fn set_text_display_text(&mut self, id: &str, text: impl Into<String>) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.display.text = text.into();
            display.scroll = 0;
            true
        } else {
            false
        }
    }

    pub fn read_text_input(&self, id: &str) -> Option<String> {
        match self.element_by_id(id) {
            Some(RuntimeElement::TextInput(input)) => Some(input.field.text.clone()),
            _ => None,
        }
    }

    pub fn set_text_input_text(&mut self, id: &str, text: impl Into<String>) -> bool {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = text.into();
            input.cursor = input.field.text.chars().count();
            input.selection_anchor = None;
            self.invalidate_text_input_layout_cache(id);
            self.recompute_all_relative_locations();
            true
        } else {
            false
        }
    }

    pub fn set_text_input_lock_status(&mut self, id: &str, locked: bool) -> bool {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.locked = locked;
            if locked {
                input.selection_anchor = None;
            }
            true
        } else {
            false
        }
    }

    pub(super) fn element_by_id(&self, id: &str) -> Option<&RuntimeElement> {
        self.elements.get(id)
    }

    pub(super) fn element_mut_by_id(&mut self, id: &str) -> Option<&mut RuntimeElement> {
        self.elements.get_mut(id)
    }

}
