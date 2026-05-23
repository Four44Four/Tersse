use tersse::pure::focus_order::{next_index, normalize_index, prev_index, sorted_ids};

#[test]
fn sorted_ids_uses_focus_then_id() {
    let sorted = sorted_ids(vec![
        (2, "z".to_string()),
        (0, "b".to_string()),
        (0, "a".to_string()),
        (1, "m".to_string()),
    ]);
    assert_eq!(sorted, vec!["a", "b", "m", "z"]);
}

#[test]
fn normalize_index_handles_empty_and_out_of_bounds() {
    assert_eq!(normalize_index(3, 0), 0);
    assert_eq!(normalize_index(10, 3), 2);
    assert_eq!(normalize_index(1, 3), 1);
}

#[test]
fn next_index_wraps_like_right_or_down_navigation() {
    assert_eq!(next_index(0, 0), 0);
    assert_eq!(next_index(0, 3), 1);
    assert_eq!(next_index(2, 3), 0);
}

#[test]
fn prev_index_wraps_like_left_or_up_navigation() {
    assert_eq!(prev_index(0, 0), 0);
    assert_eq!(prev_index(2, 3), 1);
    assert_eq!(prev_index(0, 3), 2);
}
