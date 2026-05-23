use tersse::runtime::{
    runtime_render_height_for_button, runtime_render_height_for_text_display,
    runtime_render_height_for_text_input_text,
};

#[test]
fn render_height_for_button_is_one_row() {
    assert_eq!(runtime_render_height_for_button(), 1);
}

#[test]
fn render_height_for_text_input_grows_with_wrapped_lines() {
    assert_eq!(runtime_render_height_for_text_input_text("abcde", 3), 2);
}

#[test]
fn render_height_for_text_display_uses_fixed_viewport_height() {
    assert_eq!(runtime_render_height_for_text_display(4), 4);
    assert_eq!(runtime_render_height_for_text_display(0), 1);
}
