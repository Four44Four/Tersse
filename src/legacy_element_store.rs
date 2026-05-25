//! Legacy element store used only by integration tests (`test-api` feature).

use crate::element_id::ElementId;
use crate::pure::element_id::allocate_element_id;
use crate::pure::focus_store::IndexedFocusStore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputProperty {
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub width: usize,
    pub text: String,
    pub focused: bool,
    pub text_input: Option<TextInputProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FocusError {
    IdNotFound { id: ElementId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeleteElementError {
    IdNotFound { id: ElementId },
    NoFocusedElement,
}

/// A TUI element with stable id and focus-order key.
pub struct StoredElement {
    id: usize,
    pub focus_number: f64,
    pub element: Element,
}

impl StoredElement {
    fn new(id: usize, focus_number: f64, element: Element) -> Self {
        Self {
            id,
            focus_number,
            element,
        }
    }

    pub fn id(&self) -> ElementId {
        ElementId::from_internal(self.id)
    }
}

/// Elements keyed by id with focus-order indexing.
#[derive(Default)]
pub struct ElementStore {
    store: IndexedFocusStore<StoredElement>,
    next_element_id: usize,
}

impl ElementStore {
    pub fn new() -> Self {
        Self {
            store: IndexedFocusStore::new(),
            next_element_id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.store.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.store.iter().next().is_none()
    }

    fn allocate_id(&mut self) -> usize {
        loop {
            let id = allocate_element_id(&mut self.next_element_id);
            if !self.store.contains_id(id) {
                return id;
            }
        }
    }

    fn upsert_at(&mut self, id: usize, focus_number: f64, element: Element) {
        self.store.upsert(
            id,
            focus_number,
            StoredElement::new(id, focus_number, element),
        );
    }

    /// Inserts a new element and returns its assigned id.
    pub fn insert(&mut self, focus_number: f64, element: Element) -> ElementId {
        let id = self.allocate_id();
        self.upsert_at(id, focus_number, element);
        ElementId::from_internal(id)
    }

    /// Replaces an existing element's data without changing its id.
    pub fn update(&mut self, id: ElementId, focus_number: f64, element: Element) -> bool {
        if !self.store.contains_id(id.as_internal()) {
            return false;
        }
        self.upsert_at(id.as_internal(), focus_number, element);
        true
    }

    pub fn remove(&mut self, id: ElementId) -> Option<StoredElement> {
        self.store.remove(id.as_internal())
    }

    pub fn set_focus_number(&mut self, id: ElementId, focus_number: f64) -> bool {
        self.store
            .set_focus_number(id.as_internal(), focus_number, |stored, next| {
                stored.focus_number = next;
            })
    }

    pub fn get(&self, id: ElementId) -> Option<&StoredElement> {
        self.store.get(id.as_internal())
    }

    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut StoredElement> {
        self.store.get_mut(id.as_internal())
    }

    pub fn iter(&self) -> impl Iterator<Item = &StoredElement> {
        self.store.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut StoredElement> {
        self.store.iter_mut()
    }

    pub fn focus_order_ids(&self) -> Vec<ElementId> {
        self.store
            .focus_order_ids()
            .into_iter()
            .map(ElementId::from_internal)
            .collect()
    }

    #[cfg(debug_assertions)]
    pub fn set_next_element_id_for_tests(&mut self, next_element_id: usize) {
        self.next_element_id = next_element_id;
    }
}
