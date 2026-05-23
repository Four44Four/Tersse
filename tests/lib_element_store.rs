use tersse::{
    create_text_input_field_element, delete_focused_tui_element, delete_tui_element,
    force_focus_on_element, Element, ElementStore, FocusError, StoredElement,
};

#[test]
fn element_store_orders_by_focus_number_then_id() {
    let mut store = ElementStore::new();
    store.upsert(StoredElement::new(
        "b",
        1.0,
        Element::TextInputField(create_text_input_field_element(10)),
    ));
    store.upsert(StoredElement::new(
        "a",
        0.0,
        Element::TextInputField(create_text_input_field_element(10)),
    ));
    store.upsert(StoredElement::new(
        "c",
        2.0,
        Element::TextInputField(create_text_input_field_element(10)),
    ));

    assert_eq!(store.focus_order_ids(), vec!["a", "b", "c"]);
}

#[test]
fn force_focus_on_element_targets_id() {
    let mut store = ElementStore::new();
    store.upsert(StoredElement::new(
        "first",
        0.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));
    store.upsert(StoredElement::new(
        "second",
        1.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));

    force_focus_on_element(&mut store, "second").expect("second exists");
    assert!(matches!(
        store.get("second").map(|s| &s.element),
        Some(Element::TextInputField(field)) if field.focused
    ));
    assert!(matches!(
        store.get("first").map(|s| &s.element),
        Some(Element::TextInputField(field)) if !field.focused
    ));
}

#[test]
fn delete_tui_element_and_focused_element_use_ids() {
    let mut store = ElementStore::new();
    store.upsert(StoredElement::new(
        "keep",
        0.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));
    store.upsert(StoredElement::new(
        "drop",
        1.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));

    let removed = delete_tui_element(&mut store, "drop").expect("drop exists");
    assert_eq!(removed.id, "drop");
    assert_eq!(store.len(), 1);

    force_focus_on_element(&mut store, "keep").expect("keep exists");
    let focused_removed =
        delete_focused_tui_element(&mut store).expect("focused element removed");
    assert_eq!(focused_removed.id, "keep");
    assert!(store.is_empty());
}

#[test]
fn force_focus_reports_missing_id() {
    let mut store = ElementStore::new();
    let err = force_focus_on_element(&mut store, "missing").unwrap_err();
    assert!(matches!(err, FocusError::IdNotFound { id } if id == "missing"));
}

#[test]
fn set_focus_number_reorders_in_log_time() {
    let mut store = ElementStore::new();
    store.upsert(StoredElement::new(
        "a",
        0.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));
    store.upsert(StoredElement::new(
        "b",
        2.0,
        Element::TextInputField(create_text_input_field_element(5)),
    ));

    assert!(store.set_focus_number("b", 0.5));
    assert_eq!(store.focus_order_ids(), vec!["a", "b"]);
}
