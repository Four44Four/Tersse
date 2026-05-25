/// Resolves the focus index after the focus-ordered id list changes.
///
/// When `focused_id` is still present, returns its new index. Otherwise falls back to
/// [`normalize_index`] on `fallback_index`.
pub(crate) fn index_for_focused_id(
    order: &[usize],
    focused_id: Option<usize>,
    fallback_index: usize,
) -> usize {
    if let Some(id) = focused_id {
        if let Some(pos) = order.iter().position(|entry| *entry == id) {
            return pos;
        }
    }
    normalize_index(fallback_index, order.len())
}

/// Keeps focus index valid after element list changes.
pub(crate) fn normalize_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if current >= len {
        len - 1
    } else {
        current
    }
}

/// Moves focus forward, staying on the last element when already there.
pub(crate) fn next_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if current >= len - 1 {
        len - 1
    } else {
        current + 1
    }
}

/// Element ids to redraw after keyboard input: the focused element, plus the
/// previously focused element when focus changed.
pub(crate) fn keyboard_redraw_element_ids(previous: Option<usize>, current: Option<usize>) -> Vec<usize> {
    match (previous, current) {
        (Some(prev), Some(cur)) if prev != cur => vec![prev, cur],
        (_, Some(cur)) => vec![cur],
        _ => Vec::new(),
    }
}

/// Moves focus backward, staying on the first element when already there.
pub(crate) fn prev_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if current == 0 {
        0
    } else {
        current - 1
    }
}
