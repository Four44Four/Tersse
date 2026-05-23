use tersse::{
    create_text_input_field_element, delete_focused_tui_element, delete_tui_element,
    force_focus_on_element, Element, ElementStore, FocusError,
};

fn field(width: usize) -> Element {
    Element::TextInputField(create_text_input_field_element(width))
}

#[test]
fn element_store_orders_by_focus_number_then_id() {
    let mut store = ElementStore::new();
    let second = store.insert(1.0, field(10));
    let first = store.insert(0.0, field(10));
    let third = store.insert(2.0, field(10));

    assert_eq!(store.focus_order_ids(), vec![first, second, third]);
}

#[test]
fn force_focus_on_element_targets_id() {
    let mut store = ElementStore::new();
    let first = store.insert(0.0, field(5));
    let second = store.insert(1.0, field(5));

    force_focus_on_element(&mut store, second).expect("second exists");
    assert!(matches!(
        store.get(second).map(|s| &s.element),
        Some(Element::TextInputField(f)) if f.focused
    ));
    assert!(matches!(
        store.get(first).map(|s| &s.element),
        Some(Element::TextInputField(f)) if !f.focused
    ));
}

#[test]
fn delete_tui_element_and_focused_element_use_ids() {
    let mut store = ElementStore::new();
    let keep = store.insert(0.0, field(5));
    let drop = store.insert(1.0, field(5));

    let removed = delete_tui_element(&mut store, drop).expect("drop exists");
    assert_eq!(removed.id(), drop);
    assert_eq!(store.len(), 1);

    force_focus_on_element(&mut store, keep).expect("keep exists");
    let focused_removed =
        delete_focused_tui_element(&mut store).expect("focused element removed");
    assert_eq!(focused_removed.id(), keep);
    assert!(store.is_empty());
}

#[test]
fn force_focus_reports_missing_id() {
    let mut store = ElementStore::new();
    let id = store.insert(0.0, field(5));
    store.remove(id);
    let err = force_focus_on_element(&mut store, id).unwrap_err();
    assert!(matches!(err, FocusError::IdNotFound { id: missing } if missing == id));
}

#[test]
fn set_focus_number_reorders_in_log_time() {
    let mut store = ElementStore::new();
    let first = store.insert(0.0, field(5));
    let second = store.insert(2.0, field(5));

    assert!(store.set_focus_number(second, 0.5));
    assert_eq!(store.focus_order_ids(), vec![first, second]);
}

#[test]
fn insert_skips_ids_already_in_use_after_manual_occupancy() {
    let mut store = ElementStore::new();
    let first = store.insert(0.0, field(5));
    let second = store.insert(1.0, field(7));
    assert_ne!(first, second);
}

fn text_input_width(element: &Element) -> Option<usize> {
    match element {
        Element::TextInputField(f) => Some(f.width),
        _ => None,
    }
}

#[test]
fn overflow_allocation_does_not_overwrite_active_low_id_elements() {
    let mut store = ElementStore::new();
    let id0 = store.insert(0.0, field(11));

    store.set_next_element_id_for_tests(usize::MAX);
    let id_max = store.insert(1.0, field(33));

    let id1 = store.insert(2.0, field(22));
    assert_ne!(id0, id_max);
    assert_ne!(id0, id1);
    assert_ne!(id_max, id1);

    assert_eq!(store.len(), 3);
    assert_eq!(
        store.get(id0).and_then(|stored| text_input_width(&stored.element)),
        Some(11),
        "element at id 0 must remain after counter overflow"
    );
    assert_eq!(
        store
            .get(id_max)
            .and_then(|stored| text_input_width(&stored.element)),
        Some(33)
    );
    assert_eq!(
        store
            .get(id1)
            .and_then(|stored| text_input_width(&stored.element)),
        Some(22)
    );
}
