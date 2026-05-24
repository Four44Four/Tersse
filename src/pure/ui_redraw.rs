use std::time::Instant;

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
