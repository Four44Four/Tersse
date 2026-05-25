use crate::pure::focus_store::IndexedFocusStore;

use super::types::RuntimeElement;

pub(super) struct ElementStore {
    store: IndexedFocusStore<RuntimeElement>,
}

impl ElementStore {
    pub fn new() -> Self {
        Self {
            store: IndexedFocusStore::new(),
        }
    }

    pub fn contains_id(&self, id: usize) -> bool {
        self.store.contains_id(id)
    }

    pub fn upsert(&mut self, element: RuntimeElement) {
        let id = element.id();
        let focus_number = element.focus_number();
        self.store.upsert(id, focus_number, element);
    }

    pub fn remove(&mut self, id: usize) -> Option<RuntimeElement> {
        self.store.remove(id)
    }

    pub fn set_focus_number(&mut self, id: usize, focus_number: f64) -> bool {
        if self.get(id).is_some_and(|element| element.unfocusable) {
            return false;
        }
        self.store
            .set_focus_number(id, focus_number, |element, next| {
                element.set_focus_number(next);
            })
    }

    pub fn get(&self, id: usize) -> Option<&RuntimeElement> {
        self.store.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut RuntimeElement> {
        self.store.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RuntimeElement> {
        self.store.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RuntimeElement> {
        self.store.iter_mut()
    }

    pub fn focus_order_ids(&self) -> Vec<usize> {
        self.store.focus_order_ids()
    }

    pub fn focusable_order_ids(&self) -> Vec<usize> {
        self.store
            .focus_order_ids()
            .into_iter()
            .filter(|id| !self.get(*id).is_some_and(|element| element.unfocusable))
            .collect()
    }
}
