use std::collections::{BTreeMap, HashMap};

use crate::pure::focus_key::FocusKey;
use crate::pure::focus_store::{btree_get, btree_get_mut, btree_rekey, btree_remove, btree_upsert};

use super::types::RuntimeElement;

pub(super) struct ElementStore {
    by_key: BTreeMap<FocusKey, RuntimeElement>,
    id_to_key: HashMap<String, FocusKey>,
}

impl ElementStore {
    pub fn new() -> Self {
        Self {
            by_key: BTreeMap::new(),
            id_to_key: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, element: RuntimeElement) {
        let id = element.id().to_string();
        let key = FocusKey::new(element.focus_number(), &id);
        btree_upsert(&mut self.by_key, &mut self.id_to_key, key, element);
    }

    pub fn remove(&mut self, id: &str) -> Option<RuntimeElement> {
        btree_remove(&mut self.by_key, &mut self.id_to_key, id)
    }

    pub fn set_focus_number(&mut self, id: &str, focus_number: f64) -> bool {
        if !btree_rekey(&mut self.by_key, &mut self.id_to_key, id, focus_number) {
            return false;
        }
        if let Some(element) = btree_get_mut(&mut self.by_key, &self.id_to_key, id) {
            element.set_focus_number(focus_number);
        }
        true
    }

    pub fn get(&self, id: &str) -> Option<&RuntimeElement> {
        btree_get(&self.by_key, &self.id_to_key, id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut RuntimeElement> {
        btree_get_mut(&mut self.by_key, &mut self.id_to_key, id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RuntimeElement> {
        self.by_key.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RuntimeElement> {
        self.by_key.values_mut()
    }

    pub fn focus_order_ids(&self) -> Vec<String> {
        self.by_key
            .values()
            .map(|element| element.id().to_string())
            .collect()
    }
}
