//! Parent-relative positioning and overlap resolution (no I/O).

use crate::Location;

/// Which side of the parent element a child is anchored to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParentSide {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

/// Axis-aligned bounds in terminal cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementBounds {
    pub x: u16,
    pub y: u16,
    pub width: usize,
    pub height: usize,
}

/// Placement relative to a parent element or the terminal origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementPlacement {
    pub parent_id: Option<String>,
    pub side: ParentSide,
    pub offset: Location,
}

impl ElementPlacement {
    pub fn absolute(offset: Location) -> Self {
        Self {
            parent_id: None,
            side: ParentSide::Bottom,
            offset,
        }
    }

    pub fn relative_to(
        parent_id: impl Into<String>,
        side: ParentSide,
        offset: Location,
    ) -> Self {
        Self {
            parent_id: Some(parent_id.into()),
            side,
            offset,
        }
    }

    pub fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// Default child top-left before applying `offset`, per specification.
pub fn default_child_location(
    parent: ElementBounds,
    child: ElementBounds,
    side: ParentSide,
) -> Location {
    let (x, y) = match side {
        ParentSide::Left => (
            parent.x.saturating_sub(child.width as u16),
            parent.y,
        ),
        ParentSide::Right => (
            parent.x.saturating_add(parent.width as u16),
            parent.y,
        ),
        ParentSide::Top => (
            parent.x,
            parent.y.saturating_sub(child.height as u16),
        ),
        ParentSide::Bottom => (
            parent.x,
            parent.y.saturating_add(parent.height as u16),
        ),
    };
    Location { x, y }
}

/// Applies placement offset to a base location.
pub fn apply_offset(base: Location, offset: Location) -> Location {
    Location {
        x: base.x.saturating_add(offset.x),
        y: base.y.saturating_add(offset.y),
    }
}

/// Resolves absolute location from optional parent bounds and placement.
pub fn resolve_absolute_location(
    placement: &ElementPlacement,
    parent: Option<ElementBounds>,
    child_size: ElementBounds,
) -> Option<Location> {
    let base = if let Some(parent_id) = &placement.parent_id {
        let _ = parent_id;
        let parent_bounds = parent?;
        default_child_location(parent_bounds, child_size, placement.side)
    } else {
        Location { x: 0, y: 0 }
    };
    Some(apply_offset(base, placement.offset))
}

pub fn rectangles_overlap(a: ElementBounds, b: ElementBounds) -> bool {
    let a_right = a.x as i32 + a.width as i32;
    let a_bottom = a.y as i32 + a.height as i32;
    let b_right = b.x as i32 + b.width as i32;
    let b_bottom = b.y as i32 + b.height as i32;
    (a.x as i32) < b_right
        && (b.x as i32) < a_right
        && (a.y as i32) < b_bottom
        && (b.y as i32) < a_bottom
}

fn is_parent_child(
    candidate_parent: Option<&str>,
    other_parent: Option<&str>,
    candidate_id: &str,
    other_id: &str,
) -> bool {
    candidate_parent == Some(other_id)
        || other_parent == Some(candidate_id)
        || candidate_id == other_id
}

/// Rows to push the lower overlapping element downward so it clears `candidate`.
pub fn overlap_push_delta(candidate: ElementBounds, other: ElementBounds) -> u16 {
    if !rectangles_overlap(candidate, other) {
        return 0;
    }
    let candidate_bottom = candidate.y as u32 + candidate.height as u32;
    let other_bottom = other.y as u32 + other.height as u32;
    if candidate.y >= other.y {
        candidate_bottom.saturating_sub(other.y as u32) as u16
    } else {
        other_bottom.saturating_sub(candidate.y as u32) as u16
    }
}

/// Adjusts `candidate.y` or returns downward push `(min_y, delta)` for overlap reflow.
///
/// Each push moves every non-excluded element at `y >= min_y` down by `delta` rows.
pub fn resolve_overlap_location(
    mut candidate: ElementBounds,
    others: &[(String, ElementBounds, Option<String>)],
    candidate_id: &str,
    candidate_parent: Option<&str>,
) -> (Location, Vec<(u16, i32)>) {
    let mut adjusted_others = others.to_vec();
    let mut shifts: Vec<(u16, i32)> = Vec::new();
    loop {
        let mut changed = false;
        for (other_id, other_bounds, other_parent) in &mut adjusted_others {
            if is_parent_child(
                candidate_parent,
                other_parent.as_deref(),
                candidate_id,
                other_id,
            ) {
                continue;
            }
            if !rectangles_overlap(candidate, *other_bounds) {
                continue;
            }
            let delta = overlap_push_delta(candidate, *other_bounds);
            if delta == 0 {
                continue;
            }
            if candidate.y > other_bounds.y {
                candidate.y = candidate.y.saturating_add(delta);
            } else {
                let min_y = other_bounds.y;
                other_bounds.y = other_bounds.y.saturating_add(delta);
                merge_push_shift(&mut shifts, min_y, delta as i32);
            }
            changed = true;
        }
        if !changed {
            break;
        }
    }
    (
        Location {
            x: candidate.x,
            y: candidate.y,
        },
        shifts,
    )
}

fn merge_push_shift(shifts: &mut Vec<(u16, i32)>, min_y: u16, delta: i32) {
    if let Some(entry) = shifts.iter_mut().find(|(y, _)| *y == min_y) {
        entry.1 = entry.1.max(delta);
    } else {
        shifts.push((min_y, delta));
    }
}

/// Collects direct child ids for each parent.
pub fn direct_children<'a>(
    parent_id: &str,
    placements: impl Iterator<Item = (&'a str, &'a ElementPlacement)>,
) -> Vec<String> {
    placements
        .filter_map(|(id, placement)| {
            if placement.parent_id.as_deref() == Some(parent_id) {
                Some(id.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Collects all descendant ids (depth-first).
pub fn descendant_ids(
    root_id: &str,
    placements: &[(String, ElementPlacement)],
) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = direct_children(
        root_id,
        placements
            .iter()
            .map(|(id, p)| (id.as_str(), p)),
    );
    while let Some(id) = stack.pop() {
        out.push(id.clone());
        stack.extend(direct_children(
            &id,
            placements
                .iter()
                .map(|(pid, p)| (pid.as_str(), p)),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x: u16, y: u16, width: usize, height: usize) -> ElementBounds {
        ElementBounds {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn bottom_side_places_child_below_parent() {
        let parent = bounds(0, 2, 5, 1);
        let child = bounds(0, 0, 8, 1);
        let loc = default_child_location(parent, child, ParentSide::Bottom);
        assert_eq!(loc, Location { x: 0, y: 3 });
    }

    #[test]
    fn left_side_places_child_to_the_left() {
        let parent = bounds(10, 2, 5, 1);
        let child = bounds(0, 0, 4, 1);
        let loc = default_child_location(parent, child, ParentSide::Left);
        assert_eq!(loc, Location { x: 6, y: 2 });
    }

    #[test]
    fn absolute_placement_uses_terminal_origin_plus_offset() {
        let placement = ElementPlacement::absolute(Location { x: 1, y: 2 });
        let loc = resolve_absolute_location(
            &placement,
            None,
            bounds(0, 0, 5, 1),
        )
        .unwrap();
        assert_eq!(loc, Location { x: 1, y: 2 });
    }

    #[test]
    fn relative_placement_uses_parent_bounds() {
        let placement = ElementPlacement::relative_to("p", ParentSide::Bottom, Location::default());
        let loc = resolve_absolute_location(
            &placement,
            Some(bounds(0, 4, 20, 1)),
            bounds(0, 0, 80, 12),
        )
        .unwrap();
        assert_eq!(loc, Location { x: 0, y: 5 });
    }

    #[test]
    fn overlap_resolution_pushes_existing_lower_element() {
        let candidate = bounds(0, 3, 8, 1);
        let others = vec![(
            "bar".to_string(),
            bounds(0, 3, 5, 1),
            None,
        )];
        let (loc, shifts) = resolve_overlap_location(candidate, &others, "flash", Some("foo"));
        assert_eq!(loc, Location { x: 0, y: 3 });
        assert_eq!(shifts, vec![(3, 1)]);
    }
}
