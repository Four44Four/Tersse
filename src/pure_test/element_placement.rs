pub use crate::pure::element_placement::*;

/// Constructs an element id for pure placement tests.
pub fn element_id_from_internal(id: usize) -> crate::ElementId {
    crate::ElementId::from_internal(id)
}
