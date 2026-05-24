use tersse::{runtime_clamp_fixed_height, runtime_text_input_state_snapshot};

#[test]
fn fixed_height_is_clamped_to_minimum_one() {
    assert_eq!(runtime_clamp_fixed_height(0), 1);
    assert_eq!(runtime_clamp_fixed_height(4), 4);
}

#[test]
fn runtime_text_input_state_snapshot_matches_input_values() {
    let snapshot = runtime_text_input_state_snapshot("hello", 3, Some(1));
    assert_eq!(snapshot.text, "hello");
    assert_eq!(snapshot.cursor, 3);
    assert_eq!(snapshot.selection_anchor, Some(1));
}
