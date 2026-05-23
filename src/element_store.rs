//! Focus-ordered storage for public [`crate::Element`] values.

use std::collections::{BTreeMap, HashMap};

use crate::pure::focus_key::FocusKey;
use crate::pure::focus_store::{btree_get, btree_get_mut, btree_rekey, btree_remove, btree_upsert};
use crate::Element;

/// A TUI element with stable id and focus-order key.
pub struct StoredElement {
    pub id: String,
    pub focus_number: f64,
    pub element: Element,
}

impl StoredElement {
    pub fn new(id: impl Into<String>, focus_number: f64, element: Element) -> Self {
        Self {
            id: id.into(),
            focus_number,
            element,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Elements sorted by `(focus_number, id)` with `O(log n)` insert, delete, and reorder.
#[derive(Default)]
pub struct ElementStore {
    by_key: BTreeMap<FocusKey, StoredElement>,
    id_to_key: HashMap<String, FocusKey>,
}

impl ElementStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }

    pub fn upsert(&mut self, stored: StoredElement) {
        let key = FocusKey::new(stored.focus_number, &stored.id);
        btree_upsert(&mut self.by_key, &mut self.id_to_key, key, stored);
    }

    pub fn remove(&mut self, id: &str) -> Option<StoredElement> {
        btree_remove(&mut self.by_key, &mut self.id_to_key, id)
    }

    pub fn set_focus_number(&mut self, id: &str, focus_number: f64) -> bool {
        if !btree_rekey(&mut self.by_key, &mut self.id_to_key, id, focus_number) {
            return false;
        }
        if let Some(stored) = btree_get_mut(&mut self.by_key, &self.id_to_key, id) {
            stored.focus_number = focus_number;
        }
        true
    }

    pub fn get(&self, id: &str) -> Option<&StoredElement> {
        btree_get(&self.by_key, &self.id_to_key, id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut StoredElement> {
        btree_get_mut(&mut self.by_key, &mut self.id_to_key, id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StoredElement> {
        self.by_key.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut StoredElement> {
        self.by_key.values_mut()
    }

    pub fn focus_order_ids(&self) -> Vec<String> {
        self.by_key
            .values()
            .map(|stored| stored.id.clone())
            .collect()
    }
}
