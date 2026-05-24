use std::collections::BTreeMap;

use ahash::AHashMap;

use super::focus_key::FocusKey;

/// Shared element/focus-index storage:
/// - O(1) average id access via AHashMap
/// - O(log n) ordered traversal via BTreeMap keyed by `(focus_number, id)`
pub struct IndexedFocusStore<V> {
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
    pub fn new() -> Self {
        Self {
            elements: AHashMap::new(),
            by_key: BTreeMap::new(),
            id_to_key: AHashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn contains_id(&self, id: usize) -> bool {
        self.elements.contains_key(&id)
    }

    pub fn upsert(&mut self, id: usize, focus_number: f64, value: V) {
        if let Some(old_key) = self.id_to_key.remove(&id) {
            self.by_key.remove(&old_key);
        }
        let new_key = FocusKey::new(focus_number, id);
        self.id_to_key.insert(id, new_key.clone());
        self.by_key.insert(new_key, id);
        self.elements.insert(id, value);
    }

    pub fn remove(&mut self, id: usize) -> Option<V> {
        let old_key = self.id_to_key.remove(&id)?;
        self.by_key.remove(&old_key);
        self.elements.remove(&id)
    }

    pub fn set_focus_number<F>(&mut self, id: usize, focus_number: f64, mut set_value_focus: F) -> bool
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

    pub fn get(&self, id: usize) -> Option<&V> {
        self.elements.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut V> {
        self.elements.get_mut(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.elements.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.elements.values_mut()
    }

    pub fn focus_order_ids(&self) -> Vec<usize> {
        self.by_key.values().copied().collect()
    }
}

/// Inserts or replaces an element keyed by `(focus_number, id)` in `O(log n)` time.
pub fn btree_upsert<V>(
    by_key: &mut BTreeMap<FocusKey, V>,
    id_to_key: &mut AHashMap<usize, FocusKey>,
    key: FocusKey,
    value: V,
) {
    if let Some(old_key) = id_to_key.remove(&key.id) {
        by_key.remove(&old_key);
    }
    id_to_key.insert(key.id, key.clone());
    by_key.insert(key, value);
}

/// Removes an element by id in `O(log n)` time.
pub fn btree_remove<V>(
    by_key: &mut BTreeMap<FocusKey, V>,
    id_to_key: &mut AHashMap<usize, FocusKey>,
    id: usize,
) -> Option<V> {
    let key = id_to_key.remove(&id)?;
    by_key.remove(&key)
}

/// Moves an element to a new `focus_number` in `O(log n)` time.
pub fn btree_rekey<V>(
    by_key: &mut BTreeMap<FocusKey, V>,
    id_to_key: &mut AHashMap<usize, FocusKey>,
    id: usize,
    new_focus_number: f64,
) -> bool {
    let Some(old_key) = id_to_key.get(&id).cloned() else {
        return false;
    };
    let new_key = FocusKey::new(new_focus_number, id);
    if old_key == new_key {
        return true;
    }
    let Some(value) = by_key.remove(&old_key) else {
        id_to_key.remove(&id);
        return false;
    };
    id_to_key.insert(id, new_key.clone());
    by_key.insert(new_key, value);
    true
}

/// Looks up a value by id in `O(log n)` time.
pub fn btree_get<'a, V>(
    by_key: &'a BTreeMap<FocusKey, V>,
    id_to_key: &AHashMap<usize, FocusKey>,
    id: usize,
) -> Option<&'a V> {
    let key = id_to_key.get(&id)?;
    by_key.get(key)
}

/// Looks up a mutable value by id in `O(log n)` time.
pub fn btree_get_mut<'a, V>(
    by_key: &'a mut BTreeMap<FocusKey, V>,
    id_to_key: &AHashMap<usize, FocusKey>,
    id: usize,
) -> Option<&'a mut V> {
    let key = id_to_key.get(&id)?.clone();
    by_key.get_mut(&key)
}
