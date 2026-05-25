use tersse::pure_test::button::{padding_cols, truncate_label};

#[test]
fn truncates_long_label() {
    assert_eq!(truncate_label("abcdef", 3), "abc");
}

#[test]
fn padding_fills_short_label() {
    assert_eq!(padding_cols("ab", 5), 3);
    assert_eq!(padding_cols("hello", 5), 0);
}

#[test]
fn zero_width_inputs_are_treated_as_single_column() {
    assert_eq!(truncate_label("abcdef", 0), "a");
    assert_eq!(padding_cols("ab", 0), 0);
}
