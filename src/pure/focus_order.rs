/// Returns element ids sorted by `(focus_number, id)`.
pub fn sorted_ids(mut entries: Vec<(f64, String)>) -> Vec<String> {
    entries.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, id)| id).collect()
}

/// Resolves the focus index after the focus-ordered id list changes.
///
/// When `focused_id` is still present, returns its new index. Otherwise falls back to
/// [`normalize_index`] on `fallback_index`.
pub fn index_for_focused_id(
    order: &[impl AsRef<str>],
    focused_id: Option<&str>,
    fallback_index: usize,
) -> usize {
    if let Some(id) = focused_id {
        if let Some(pos) = order.iter().position(|entry| entry.as_ref() == id) {
            return pos;
        }
    }
    normalize_index(fallback_index, order.len())
}

/// Keeps focus index valid after element list changes.
pub fn normalize_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if current >= len {
        len - 1
    } else {
        current
    }
}

/// Moves focus forward with wraparound.
pub fn next_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (current + 1) % len
    }
}

/// Moves focus backward with wraparound.
pub fn prev_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if current == 0 {
        len - 1
    } else {
        current - 1
    }
}
