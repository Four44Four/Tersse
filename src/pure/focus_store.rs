use std::collections::BTreeMap;

use ahash::AHashMap;

use super::focus_key::FocusKey;

/// Shared element/focus-index storage:
/// - O(1) average id access via AHashMap
/// - O(log n) ordered traversal via BTreeMap keyed by `(focus_number, id)`
pub(crate) struct IndexedFocusStore<V> {
    elements: AHashMap<usize, V>,
    by_key: BTreeMap<FocusKey, usize>,
    id_to_key: AHashMap<usize, FocusKey>,
}

impl<V> Default for IndexedFocusStore<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> IndexedFocusStore<V> {
    pub(crate) fn new() -> Self {
        Self {
            elements: AHashMap::new(),
            by_key: BTreeMap::new(),
            id_to_key: AHashMap::new(),
        }
    }

    pub(crate) fn contains_id(&self, id: usize) -> bool {
        self.elements.contains_key(&id)
    }

    pub(crate) fn upsert(&mut self, id: usize, focus_number: f64, value: V) {
        if let Some(old_key) = self.id_to_key.remove(&id) {
            self.by_key.remove(&old_key);
        }
        let new_key = FocusKey::new(focus_number, id);
        self.id_to_key.insert(id, new_key.clone());
        self.by_key.insert(new_key, id);
        self.elements.insert(id, value);
    }

    pub(crate) fn remove(&mut self, id: usize) -> Option<V> {
        let old_key = self.id_to_key.remove(&id)?;
        self.by_key.remove(&old_key);
        self.elements.remove(&id)
    }

    pub(crate) fn set_focus_number<F>(
        &mut self,
        id: usize,
        focus_number: f64,
        mut set_value_focus: F,
    ) -> bool
    where
        F: FnMut(&mut V, f64),
    {
        let Some(value) = self.elements.get_mut(&id) else {
            return false;
        };
        let Some(old_key) = self.id_to_key.remove(&id) else {
            return false;
        };
        self.by_key.remove(&old_key);
        set_value_focus(value, focus_number);
        let new_key = FocusKey::new(focus_number, id);
        self.id_to_key.insert(id, new_key.clone());
        self.by_key.insert(new_key, id);
        true
    }

    pub(crate) fn get(&self, id: usize) -> Option<&V> {
        self.elements.get(&id)
    }

    pub(crate) fn get_mut(&mut self, id: usize) -> Option<&mut V> {
        self.elements.get_mut(&id)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &V> {
        self.elements.values()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.elements.values_mut()
    }

    pub(crate) fn focus_order_ids(&self) -> Vec<usize> {
        self.by_key.values().copied().collect()
    }
}
