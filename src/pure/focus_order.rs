/// Returns element ids sorted by `(focus_number, id)`.
pub fn sorted_ids(mut entries: Vec<(f64, String)>) -> Vec<String> {
    entries.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, id)| id).collect()
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
