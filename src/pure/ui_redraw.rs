use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementLocationSnapshot {
    pub id: usize,
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutRedrawDecision {
    /// Redraw only these elements (location and/or appearance changed, no unrelated reflow).
    Elements,
    /// Unrelated elements were pushed/pulled; redraw from this y downward.
    CascadeFromY,
}

/// Returns which elements to repaint after a layout mutation.
///
/// `primary_id` is the element that was created, removed, moved, or resized. Descendants that
/// move with it are not treated as reflow of unrelated elements.
pub fn layout_redraw_decision(
    primary_id: usize,
    before: &[ElementLocationSnapshot],
    after: &[ElementLocationSnapshot],
    parent_links: &[(usize, Option<usize>)],
) -> (LayoutRedrawDecision, Vec<usize>, u16) {
    let before_map: HashMap<usize, (u16, u16)> = before
        .iter()
        .map(|entry| (entry.id, (entry.x, entry.y)))
        .collect();
    let after_map: HashMap<usize, (u16, u16)> = after
        .iter()
        .map(|entry| (entry.id, (entry.x, entry.y)))
        .collect();

    let anchor_y = before_map
        .get(&primary_id)
        .map(|(_, y)| *y)
        .or_else(|| after_map.get(&primary_id).map(|(_, y)| *y))
        .unwrap_or(0);

    let mut redraw_ids = Vec::new();
    for entry in after {
        let changed = before_map
            .get(&entry.id)
            .is_none_or(|(bx, by)| (entry.x, entry.y) != (*bx, *by));
        if changed {
            redraw_ids.push(entry.id);
        }
    }

    if other_elements_reflowed(primary_id, &before_map, &after_map, parent_links) {
        (
            LayoutRedrawDecision::CascadeFromY,
            redraw_ids,
            anchor_y,
        )
    } else {
        (LayoutRedrawDecision::Elements, redraw_ids, anchor_y)
    }
}

fn other_elements_reflowed(
    primary_id: usize,
    before: &HashMap<usize, (u16, u16)>,
    after: &HashMap<usize, (u16, u16)>,
    parent_links: &[(usize, Option<usize>)],
) -> bool {
    for (id, (ax, ay)) in after {
        if *id == primary_id {
            continue;
        }
        if is_descendant_of(*id, primary_id, parent_links) {
            continue;
        }
        let Some((bx, by)) = before.get(id) else {
            return true;
        };
        if (ax, ay) != (bx, by) {
            return true;
        }
    }
    for id in before.keys() {
        if *id == primary_id {
            continue;
        }
        if is_descendant_of(*id, primary_id, parent_links) {
            continue;
        }
        if !after.contains_key(id) {
            return true;
        }
    }
    false
}

fn is_descendant_of(
    element_id: usize,
    ancestor_id: usize,
    parent_links: &[(usize, Option<usize>)],
) -> bool {
    let mut current = Some(element_id);
    while let Some(id) = current {
        if id == ancestor_id {
            return true;
        }
        current = parent_of(id, parent_links);
    }
    false
}

fn parent_of(child: usize, parent_links: &[(usize, Option<usize>)]) -> Option<usize> {
    parent_links
        .iter()
        .find(|(id, _)| *id == child)
        .and_then(|(_, parent)| *parent)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementRedrawPlan {
    redraw_from_y: Option<u16>,
    exact_ids: Vec<usize>,
}

impl ElementRedrawPlan {
    pub fn is_empty(&self) -> bool {
        self.redraw_from_y.is_none() && self.exact_ids.is_empty()
    }

    pub fn clear(&mut self) {
        self.redraw_from_y = None;
        self.exact_ids.clear();
    }

    pub fn redraw_from_y(&self) -> Option<u16> {
        self.redraw_from_y
    }

    pub fn mark_element(&mut self, id: usize) {
        if !self.exact_ids.contains(&id) {
            self.exact_ids.push(id);
        }
    }

    pub fn mark_from_y(&mut self, y: u16) {
        self.redraw_from_y = Some(self.redraw_from_y.map_or(y, |current| current.min(y)));
    }

    pub fn should_draw(&self, id: usize, y: u16) -> bool {
        self.redraw_from_y.is_some_and(|from_y| y >= from_y) || self.exact_ids.contains(&id)
    }
}

pub fn should_flush_debounced_queue_redraw(
    has_pending_redraw: bool,
    queue_has_pending_work: bool,
    deadline: Option<Instant>,
    now: Instant,
) -> bool {
    if !has_pending_redraw || queue_has_pending_work {
        return false;
    }
    match deadline {
        Some(until) => now >= until,
        None => true,
    }
}
