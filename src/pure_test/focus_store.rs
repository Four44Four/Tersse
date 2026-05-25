use std::collections::BTreeMap;

use ahash::AHashMap;

use super::focus_key::FocusKey;

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
