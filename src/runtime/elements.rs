use crate::pure::layout_reflow;
use crate::Location;

use super::types::{
    ButtonConfig, ButtonElement, ElementConfig, RuntimeElement, TextDisplayConfig,
    TextDisplayRuntimeElement, TextInputConfig, TextInputElement,
};
use super::RuntimeUi;

impl RuntimeUi {
    pub fn upsert_button(&mut self, config: ButtonConfig) {
        let element = RuntimeElement::Button(ButtonElement::from_config(config));
        self.elements.upsert(element);
        self.sync_focus_position();
        self.refresh_height_cache();
    }

    pub fn button_width(&self, id: &str) -> Option<usize> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.width),
            _ => None,
        }
    }

    pub fn upsert_text_input(&mut self, config: TextInputConfig) {
        let id = config.id.clone();
        let element = RuntimeElement::TextInput(TextInputElement::from_config(config));
        self.elements.upsert(element);
        self.sync_focus_position();
        self.invalidate_text_input_layout_cache(&id);
        self.refresh_height_cache();
    }

    pub fn upsert_text_display(&mut self, config: TextDisplayConfig) {
        let element = RuntimeElement::TextDisplay(TextDisplayRuntimeElement::from_config(config));
        self.elements.upsert(element);
        self.sync_focus_position();
        self.refresh_height_cache();
    }

    pub fn upsert_and_reflow(&mut self, config: ElementConfig) {
        let id = match &config {
            ElementConfig::Button(cfg) => cfg.id.clone(),
            ElementConfig::TextInput(cfg) => cfg.id.clone(),
            ElementConfig::TextDisplay(cfg) => cfg.id.clone(),
        };
        let old_height = self.element_render_height_by_id(&id).unwrap_or(0);
        let anchor_y = self
            .element_location(&id)
            .map(|loc| loc.y)
            .or_else(|| match &config {
                ElementConfig::Button(cfg) => Some(cfg.location.y),
                ElementConfig::TextInput(cfg) => Some(cfg.location.y),
                ElementConfig::TextDisplay(cfg) => Some(cfg.location.y),
            });

        match config {
            ElementConfig::Button(cfg) => self.upsert_button(cfg),
            ElementConfig::TextInput(cfg) => self.upsert_text_input(cfg),
            ElementConfig::TextDisplay(cfg) => self.upsert_text_display(cfg),
        }

        let new_height = self.element_render_height_by_id(&id).unwrap_or(0);
        let delta = layout_reflow::height_delta(old_height, new_height);
        if delta != 0 {
            if let Some(y) = anchor_y {
                let min_y = layout_reflow::min_y_after_change(y, old_height);
                self.shift_elements_from_min_y(&id, min_y, delta);
            }
        }
        self.refresh_height_cache();
    }

    pub fn remove_and_reflow(&mut self, id: &str) -> bool {
        let Some(location) = self.element_location(id) else {
            return false;
        };
        let removed_height = self.element_render_height_by_id(id).unwrap_or(0);
        if !self.remove_element(id) {
            return false;
        }
        if removed_height > 0 {
            let min_y = layout_reflow::min_y_after_change(location.y, removed_height);
            self.shift_elements_from_min_y(id, min_y, -(removed_height as i32));
        }
        self.refresh_height_cache();
        true
    }

    pub fn remove_element(&mut self, id: &str) -> bool {
        if self.elements.remove(id).is_some() {
            self.sync_focus_position();
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
        if self.elements.set_focus_number(id, focus_number) {
            self.sync_focus_position();
            true
        } else {
            false
        }
    }

    pub fn set_element_location(&mut self, id: &str, location: Location) -> bool {
        if let Some(element) = self.element_mut_by_id(id) {
            match element {
                RuntimeElement::Button(button) => button.button.location = location,
                RuntimeElement::TextInput(input) => input.location = location,
                RuntimeElement::TextDisplay(display) => display.location = location,
            }
            true
        } else {
            false
        }
    }

    pub fn set_text_display_dimensions(&mut self, id: &str, width: usize, height: usize) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.width = width.max(1);
            display.height = height.max(1);
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
