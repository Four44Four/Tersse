//! Opaque element handle shared by the runtime and legacy test store.

/// Opaque handle for an element assigned by the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(usize);

impl ElementId {
    pub(crate) fn from_internal(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn as_internal(self) -> usize {
        self.0
    }
}
