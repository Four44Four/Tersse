/// Returns the next counter value, wrapping to `0` on `usize` overflow.
pub(crate) fn advance_element_id(current: usize) -> usize {
    current.wrapping_add(1)
}

/// Allocates the current counter value and advances it.
pub(crate) fn allocate_element_id(counter: &mut usize) -> usize {
    let id = *counter;
    *counter = advance_element_id(*counter);
    id
}
