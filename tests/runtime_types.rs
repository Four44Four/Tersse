use tersse::runtime::{
    runtime_clamp_text_display_dimensions, runtime_text_input_state_snapshot,
};

#[test]
fn text_display_dimensions_are_clamped_to_minimum_one() {
    assert_eq!(runtime_clamp_text_display_dimensions(0, 0), (1, 1));
    assert_eq!(runtime_clamp_text_display_dimensions(20, 4), (20, 4));
}

#[test]
fn runtime_text_input_state_snapshot_matches_input_values() {
    let snapshot = runtime_text_input_state_snapshot("hello", 3, Some(1));
    assert_eq!(snapshot.text, "hello");
    assert_eq!(snapshot.cursor, 3);
    assert_eq!(snapshot.selection_anchor, Some(1));
}
