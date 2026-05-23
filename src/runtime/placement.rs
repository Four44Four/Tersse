use crate::pure::element_placement::{
    descendant_ids, resolve_absolute_location, resolve_overlap_location, ElementBounds,
    ElementPlacement,
};
use crate::ElementId;
use crate::Location;

use super::layout::{
    render_height_for_button, render_height_for_text_display, render_height_for_text_input_text,
};
use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn resolve_config_location(
        &mut self,
        id: usize,
        placement: &ElementPlacement,
        width: usize,
        height: usize,
    ) -> Location {
        let candidate = self.placement_candidate_bounds(placement, width, height);
        let base = self.placement_base_location(placement, candidate);
        let candidate = ElementBounds {
            x: base.x,
            y: base.y,
            ..candidate
        };
        let others = self.collect_overlap_candidates(id);
        let (location, shifts) = resolve_overlap_location(
            candidate,
            &others,
            id,
            placement.parent_id,
        );
        for (min_y, delta) in shifts {
            self.push_elements_down_from(min_y, delta, &[id]);
        }
        location
    }

    /// Pushes root-positioned elements at `y >= min_y` down, then recomputes relative children.
    pub(super) fn push_elements_down_from(
        &mut self,
        min_y: u16,
        delta: i32,
        exclude_ids: &[usize],
    ) {
        if delta <= 0 {
            return;
        }
        let ids: Vec<usize> = self.elements.iter().map(|element| element.id()).collect();
        for id in ids {
            if exclude_ids.contains(&id) {
                continue;
            }
            let Some(placement) = self.placement_for(id) else {
                continue;
            };
            if placement.has_parent() {
                continue;
            }
            let Some(location) = self.element_location(ElementId::from_internal(id)) else {
                continue;
            };
            if location.y < min_y {
                continue;
            }
            let _ = self.set_element_location_by_id(
                id,
                Location {
                    x: location.x,
                    y: location.y.saturating_add(delta as u16),
                },
            );
        }
        self.recompute_all_relative_locations();
    }

    /// Pulls root-positioned elements at `y >= min_y` up, then recomputes relative children.
    pub(super) fn pull_elements_up_from(
        &mut self,
        min_y: u16,
        rows: u16,
        exclude_ids: &[usize],
    ) {
        if rows == 0 {
            return;
        }
        let ids: Vec<usize> = self.elements.iter().map(|element| element.id()).collect();
        for id in ids {
            if exclude_ids.contains(&id) {
                continue;
            }
            let Some(placement) = self.placement_for(id) else {
                continue;
            };
            if placement.has_parent() {
                continue;
            }
            let Some(location) = self.element_location(ElementId::from_internal(id)) else {
                continue;
            };
            if location.y < min_y {
                continue;
            }
            let _ = self.set_element_location_by_id(
                id,
                Location {
                    x: location.x,
                    y: location.y.saturating_sub(rows),
                },
            );
        }
        self.recompute_all_relative_locations();
    }

    /// Resets every element to its placement-derived location (used after removal).
    pub(super) fn relayout_all_from_placements(&mut self) {
        let ids: Vec<usize> = self.elements.iter().map(|element| element.id()).collect();
        for id in &ids {
            let Some(placement) = self.placement_for(*id) else {
                continue;
            };
            if placement.has_parent() {
                continue;
            }
            let Some((width, height)) = self.element_dimensions(*id) else {
                continue;
            };
            let location = self.resolve_placement_only(&placement, width, height);
            let _ = self.set_element_location_by_id(*id, location);
        }
        self.recompute_all_relative_locations();
    }

    pub(super) fn resolve_placement_only(
        &self,
        placement: &ElementPlacement,
        width: usize,
        height: usize,
    ) -> Location {
        let candidate = self.placement_candidate_bounds(placement, width, height);
        self.placement_base_location(placement, candidate)
    }

    pub(super) fn recompute_all_relative_locations(&mut self) {
        let count = self.elements.iter().count();
        for _ in 0..count.max(1) {
            let ids = self
                .elements
                .iter()
                .filter_map(|element| {
                    let id = element.id();
                    let placement = self.placement_for(id)?;
                    placement.has_parent().then_some(id)
                })
                .collect::<Vec<_>>();
            for id in ids {
                let Some(placement) = self.placement_for(id) else {
                    continue;
                };
                let Some((width, height)) = self.element_dimensions(id) else {
                    continue;
                };
                let location = self.resolve_placement_only(&placement, width, height);
                let _ = self.set_element_location_by_id(id, location);
            }
        }
    }

    fn placement_candidate_bounds(
        &self,
        _placement: &ElementPlacement,
        width: usize,
        height: usize,
    ) -> ElementBounds {
        ElementBounds {
            x: 0,
            y: 0,
            width: width.max(1),
            height: height.max(1),
        }
    }

    fn placement_base_location(
        &self,
        placement: &ElementPlacement,
        child_bounds: ElementBounds,
    ) -> Location {
        let parent_bounds = placement
            .parent_id
            .and_then(|parent_id| self.element_bounds(ElementId::from_internal(parent_id)));
        resolve_absolute_location(placement, parent_bounds, child_bounds)
            .unwrap_or(Location::default())
    }

    pub(super) fn element_bounds(&self, id: ElementId) -> Option<ElementBounds> {
        let element = self.element_by_id(id)?;
        Some(self.runtime_element_bounds(element))
    }

    pub(super) fn runtime_element_bounds(&self, element: &RuntimeElement) -> ElementBounds {
        match element {
            RuntimeElement::Button(button) => ElementBounds {
                x: button.button.location.x,
                y: button.button.location.y,
                width: button.button.width,
                height: render_height_for_button(),
            },
            RuntimeElement::TextInput(input) => {
                let width = input.field.width.max(1);
                let height = render_height_for_text_input_text(&input.field.text, width);
                ElementBounds {
                    x: input.location.x,
                    y: input.location.y,
                    width,
                    height,
                }
            }
            RuntimeElement::TextDisplay(display) => ElementBounds {
                x: display.location.x,
                y: display.location.y,
                width: display.width,
                height: render_height_for_text_display(display.height),
            },
        }
    }

    pub(super) fn placement_for(&self, id: usize) -> Option<ElementPlacement> {
        match self.element_by_id(ElementId::from_internal(id))? {
            RuntimeElement::Button(button) => Some(button.placement.clone()),
            RuntimeElement::TextInput(input) => Some(input.placement.clone()),
            RuntimeElement::TextDisplay(display) => Some(display.placement.clone()),
        }
    }

    pub(super) fn set_element_location_by_id(&mut self, id: usize, location: Location) -> bool {
        let Some(element) = self.element_mut_by_id(ElementId::from_internal(id)) else {
            return false;
        };
        match element {
            RuntimeElement::Button(button) => button.button.location = location,
            RuntimeElement::TextInput(input) => input.location = location,
            RuntimeElement::TextDisplay(display) => display.location = location,
        }
        true
    }

    pub(super) fn shift_element_subtree(&mut self, root_id: ElementId, delta_x: i16, delta_y: i32) {
        if delta_x == 0 && delta_y == 0 {
            return;
        }
        let root = root_id.as_internal();
        let mut ids = vec![root];
        let placements = self.all_placements();
        ids.extend(descendant_ids(root, &placements));
        for id in ids {
            let Some(location) = self.element_location(ElementId::from_internal(id)) else {
                continue;
            };
            let shifted = Location {
                x: (location.x as i32 + delta_x as i32).max(0) as u16,
                y: (location.y as i32 + delta_y).max(0) as u16,
            };
            let _ = self.set_element_location_by_id(id, shifted);
        }
    }

    pub(super) fn remove_element_cascade(&mut self, id: ElementId) -> bool {
        let placements = self.all_placements();
        let root = id.as_internal();
        let mut to_remove = descendant_ids(root, &placements);
        to_remove.push(root);
        let mut removed = false;
        for remove_id in to_remove {
            if self.remove_element(ElementId::from_internal(remove_id)) {
                removed = true;
            }
        }
        removed
    }

    fn collect_overlap_candidates(
        &self,
        skip_id: usize,
    ) -> Vec<(usize, ElementBounds, Option<usize>)> {
        self.elements
            .iter()
            .filter_map(|element| {
                let id = element.id();
                if id == skip_id {
                    return None;
                }
                let placement = match element {
                    RuntimeElement::Button(button) => button.placement.clone(),
                    RuntimeElement::TextInput(input) => input.placement.clone(),
                    RuntimeElement::TextDisplay(display) => display.placement.clone(),
                };
                Some((
                    id,
                    self.runtime_element_bounds(element),
                    placement.parent_id,
                ))
            })
            .collect()
    }

    fn all_placements(&self) -> Vec<(usize, ElementPlacement)> {
        self.elements
            .iter()
            .map(|element| {
                let placement = match element {
                    RuntimeElement::Button(button) => button.placement.clone(),
                    RuntimeElement::TextInput(input) => input.placement.clone(),
                    RuntimeElement::TextDisplay(display) => display.placement.clone(),
                };
                (element.id(), placement)
            })
            .collect()
    }

    fn element_dimensions(&mut self, id: usize) -> Option<(usize, usize)> {
        let element_id = ElementId::from_internal(id);
        match self.element_by_id(element_id)? {
            RuntimeElement::Button(button) => Some((button.button.width, render_height_for_button())),
            RuntimeElement::TextInput(input) => {
                let width = input.field.width.max(1);
                let height = self.text_input_render_height(element_id)?;
                Some((width, height))
            }
            RuntimeElement::TextDisplay(display) => Some((
                display.width,
                render_height_for_text_display(display.height),
            )),
        }
    }

    pub(super) fn button_config_dimensions(config: &super::types::ButtonConfig) -> (usize, usize) {
        (config.width.max(1), render_height_for_button())
    }

    pub(super) fn text_input_config_dimensions(
        config: &super::types::TextInputConfig,
    ) -> (usize, usize) {
        let width = config.width.max(1);
        let height = render_height_for_text_input_text(&config.initial_text, width);
        (width, height)
    }

    pub(super) fn text_display_config_dimensions(
        config: &super::types::TextDisplayConfig,
    ) -> (usize, usize) {
        let (width, height) = super::types::clamp_text_display_dimensions(config.width, config.height);
        (width, height)
    }
}
