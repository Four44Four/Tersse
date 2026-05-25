/// Returns signed row delta (`new_height - old_height`).
pub(crate) fn height_delta(old_height: usize, new_height: usize) -> i32 {
    new_height as i32 - old_height as i32
}

/// Returns minimum y that downstream elements should compare against.
pub(crate) fn min_y_after_change(anchor_y: u16, old_height: usize) -> u16 {
    if old_height == 0 {
        anchor_y
    } else {
        anchor_y.saturating_add(old_height as u16)
    }
}
