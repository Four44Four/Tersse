/// Returns the next counter value, wrapping to `0` on `usize` overflow.
pub fn advance_element_id(current: usize) -> usize {
    crate::pure::element_id::advance_element_id(current)
}

/// Allocates the current counter value and advances it.
pub fn allocate_element_id(counter: &mut usize) -> usize {
    crate::pure::element_id::allocate_element_id(counter)
}
