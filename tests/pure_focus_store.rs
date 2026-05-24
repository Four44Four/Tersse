use std::collections::BTreeMap;

use ahash::AHashMap;

use tersse::pure::focus_key::FocusKey;
use tersse::pure::focus_store::{btree_get, btree_rekey, btree_remove, btree_upsert};

#[test]
fn btree_upsert_remove_and_rekey_are_logarithmic_map_ops() {
    let mut by_key: BTreeMap<FocusKey, &'static str> = BTreeMap::new();
    let mut id_to_key: AHashMap<usize, FocusKey> = AHashMap::new();

    btree_upsert(
        &mut by_key,
        &mut id_to_key,
        FocusKey::new(1.0, 1),
        "beta",
    );
    btree_upsert(
        &mut by_key,
        &mut id_to_key,
        FocusKey::new(0.0, 0),
        "alpha",
    );
    btree_upsert(
        &mut by_key,
        &mut id_to_key,
        FocusKey::new(2.0, 2),
        "gamma",
    );

    assert_eq!(
        by_key
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec!["alpha", "beta", "gamma"]
    );
    assert_eq!(btree_get(&by_key, &id_to_key, 1), Some(&"beta"));

    assert!(btree_rekey(&mut by_key, &mut id_to_key, 1, 0.5));
    assert_eq!(
        by_key
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec!["alpha", "beta", "gamma"]
    );
    assert_eq!(
        by_key.keys().map(|key| key.id).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );

    assert_eq!(btree_remove(&mut by_key, &mut id_to_key, 1), Some("beta"));
    assert_eq!(by_key.len(), 2);
    assert!(btree_get(&by_key, &id_to_key, 1).is_none());
}
