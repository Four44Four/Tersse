/// Returns element ids sorted by `(focus_number, id)`.
pub fn sorted_ids(mut entries: Vec<(f64, usize)>) -> Vec<usize> {
    entries.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, id)| id).collect()
}

/// Returns ids from `order` that are eligible for keyboard focus.
pub fn focusable_order_ids(order: &[usize], is_unfocusable: impl Fn(usize) -> bool) -> Vec<usize> {
    order
        .iter()
        .copied()
        .filter(|id| !is_unfocusable(*id))
        .collect()
}

/// Resolves the focus index after the focus-ordered id list changes.
pub fn index_for_focused_id(
    order: &[usize],
    focused_id: Option<usize>,
    fallback_index: usize,
) -> usize {
    crate::pure::focus_order::index_for_focused_id(order, focused_id, fallback_index)
}

/// Keeps focus index valid after element list changes.
pub fn normalize_index(current: usize, len: usize) -> usize {
    crate::pure::focus_order::normalize_index(current, len)
}

/// Moves focus forward, staying on the last element when already there.
pub fn next_index(current: usize, len: usize) -> usize {
    crate::pure::focus_order::next_index(current, len)
}

/// Element ids to redraw after keyboard input.
pub fn keyboard_redraw_element_ids(previous: Option<usize>, current: Option<usize>) -> Vec<usize> {
    crate::pure::focus_order::keyboard_redraw_element_ids(previous, current)
}

/// Moves focus backward, staying on the first element when already there.
pub fn prev_index(current: usize, len: usize) -> usize {
    crate::pure::focus_order::prev_index(current, len)
}
