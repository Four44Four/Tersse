use tersse::pure::element_id::{advance_element_id, allocate_element_id};

#[test]
fn advance_element_id_wraps_on_overflow() {
    assert_eq!(advance_element_id(usize::MAX), 0);
}

#[test]
fn allocate_element_id_increments_counter() {
    let mut counter = 0;
    assert_eq!(allocate_element_id(&mut counter), 0);
    assert_eq!(allocate_element_id(&mut counter), 1);
    assert_eq!(counter, 2);
}
