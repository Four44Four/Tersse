use tersse::test_api::{runtime_clamp_fixed_height, runtime_render_height_for_element_text};

#[test]
fn render_height_for_text_input_grows_with_wrapped_lines() {
    assert_eq!(runtime_render_height_for_element_text("abcde", 3), 2);
}

#[test]
fn fixed_height_is_clamped_to_minimum_one() {
    assert_eq!(runtime_clamp_fixed_height(4), 4);
    assert_eq!(runtime_clamp_fixed_height(0), 1);
}
