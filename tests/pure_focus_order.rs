use tersse::pure::focus_order::{
    index_for_focused_id, keyboard_redraw_element_ids, next_index, normalize_index, prev_index,
    sorted_ids,
};

#[test]
fn sorted_ids_uses_focus_then_id() {
    let sorted = sorted_ids(vec![(2.0, 3), (0.0, 1), (0.0, 0), (1.0, 2)]);
    assert_eq!(sorted, vec![0, 1, 2, 3]);
}

#[test]
fn normalize_index_handles_empty_and_out_of_bounds() {
    assert_eq!(normalize_index(3, 0), 0);
    assert_eq!(normalize_index(10, 3), 2);
    assert_eq!(normalize_index(1, 3), 1);
}

#[test]
fn next_index_advances_until_last_element() {
    assert_eq!(next_index(0, 0), 0);
    assert_eq!(next_index(0, 3), 1);
    assert_eq!(next_index(1, 3), 2);
}

#[test]
fn next_index_stays_on_last_element() {
    assert_eq!(next_index(2, 3), 2);
    assert_eq!(next_index(5, 3), 2);
}

#[test]
fn index_for_focused_id_tracks_same_element_after_list_changes() {
    let order = vec![0, 1, 2];
    assert_eq!(index_for_focused_id(&order, Some(1), 0), 1);
    assert_eq!(index_for_focused_id(&order, Some(2), 0), 2);

    let after_remove_a = vec![1, 2];
    assert_eq!(index_for_focused_id(&after_remove_a, Some(2), 2), 1);

    let after_insert = vec![9, 0, 1, 2];
    assert_eq!(index_for_focused_id(&after_insert, Some(1), 1), 2);
}

#[test]
fn index_for_focused_id_falls_back_when_focused_id_is_gone() {
    let order = vec![0, 2];
    assert_eq!(index_for_focused_id(&order, Some(1), 1), 1);
    assert_eq!(index_for_focused_id(&order, None, 0), 0);
}

#[test]
fn prev_index_retreats_until_first_element() {
    assert_eq!(prev_index(0, 0), 0);
    assert_eq!(prev_index(2, 3), 1);
    assert_eq!(prev_index(1, 3), 0);
}

#[test]
fn prev_index_stays_on_first_element() {
    assert_eq!(prev_index(0, 3), 0);
}

#[test]
fn keyboard_redraw_element_ids_redraws_current_only_when_focus_unchanged() {
    assert_eq!(keyboard_redraw_element_ids(None, None), Vec::<usize>::new());
    assert_eq!(keyboard_redraw_element_ids(None, Some(1)), vec![1]);
    assert_eq!(keyboard_redraw_element_ids(Some(1), Some(1)), vec![1]);
}

#[test]
fn keyboard_redraw_element_ids_redraws_previous_and_current_on_focus_change() {
    assert_eq!(keyboard_redraw_element_ids(Some(1), Some(2)), vec![1, 2]);
    assert_eq!(keyboard_redraw_element_ids(Some(2), Some(1)), vec![2, 1]);
}
