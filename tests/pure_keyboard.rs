use tersse::pure::keyboard::arrow_extend_selection;

#[test]
fn shift_extends_selection() {
    assert!(arrow_extend_selection(true));
    assert!(!arrow_extend_selection(false));
}

#[test]
fn only_shift_changes_selection_extension_behavior() {
    let shift_held = true;
    let shift_not_held = false;
    assert!(arrow_extend_selection(shift_held));
    assert!(!arrow_extend_selection(shift_not_held));
}
