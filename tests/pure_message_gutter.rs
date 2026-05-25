use tersse::pure::message_gutter::{
    apply_message, clamp_screen_scroll_up_reveal_with_gutter, clamp_screen_scroll_with_gutter,
    clip_cols_to_avoid_wrapping_into_row, element_row_intersects_gutter_screen_rows,
    gutter_rows_to_restore, gutter_screen_rows, hide_message, layout_message_gutter_lines,
    max_screen_scroll_offset, max_screen_scroll_up_reveal, message_gutter_height,
    padding_screen_rows, ratchet_gutter_scroll_cap_on_up, ratchet_gutter_up_reveal_cap_on_down,
    row_printing_wraps_into_gutter_block, screen_scroll_shows_padding,
    screen_scroll_shows_top_padding, scroll_screen_down_with_gutter, scroll_screen_up_with_gutter,
    should_hide_gutter_by_scroll_reveal, top_padding_screen_rows,
    viewport_height_for_screen_scroll, MessageGutterState, MsgGutterSide,
};

#[test]
fn apply_message_sets_multi_indicator_when_already_visible() {
    let state = MessageGutterState {
        visible: true,
        message: "first".into(),
        show_multi_indicator: false,
        rendered_height: 1,
    };
    let next = apply_message(&state, "second", true);
    assert_eq!(next.message, "second");
    assert!(next.show_multi_indicator);
    assert!(next.visible);
}

#[test]
fn apply_message_clears_multi_indicator_when_gutter_was_hidden() {
    let state = MessageGutterState::default();
    let next = apply_message(&state, "hello", false);
    assert_eq!(next.message, "hello");
    assert!(!next.show_multi_indicator);
}

#[test]
fn hide_message_preserves_rendered_height_for_restore() {
    let state = MessageGutterState {
        visible: true,
        message: "hello".into(),
        show_multi_indicator: false,
        rendered_height: 3,
    };
    let hidden = hide_message(&state);
    assert!(!hidden.visible);
    assert_eq!(hidden.rendered_height, 3);
}

#[test]
fn multi_indicator_fits_on_last_wrapped_line() {
    let lines = layout_message_gutter_lines("hello", true, "[+]", 10);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].message_text, "hello ");
    assert_eq!(lines[0].indicator_text.as_deref(), Some("[+]"));
}

#[test]
fn multi_indicator_wraps_to_new_line_when_needed() {
    let lines = layout_message_gutter_lines("123456789", true, "[+]", 10);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].message_text, "123456789");
    assert!(lines[0].indicator_text.is_none());
    assert_eq!(lines[1].message_text, "");
    assert_eq!(lines[1].indicator_text.as_deref(), Some("[+]"));
}

#[test]
fn message_gutter_height_is_capped_by_max_height() {
    let message = "a".repeat(40);
    let height = message_gutter_height(&message, false, "[+]", 5, 3);
    assert_eq!(height, 3);
}

#[test]
fn gutter_screen_rows_top_and_bottom() {
    assert_eq!(gutter_screen_rows(MsgGutterSide::Top, 2, 23), 0..2);
    assert_eq!(gutter_screen_rows(MsgGutterSide::Bottom, 2, 23), 21..23);
    assert_eq!(gutter_screen_rows(MsgGutterSide::Bottom, 1, 23), 22..23);
}

#[test]
fn gutter_rows_to_restore_only_returns_rows_dropped_on_shrink() {
    assert_eq!(gutter_rows_to_restore(MsgGutterSide::Top, 3, 1, 10), 1..3);
    assert_eq!(
        gutter_rows_to_restore(MsgGutterSide::Bottom, 3, 1, 10),
        7..9
    );
    assert_eq!(gutter_rows_to_restore(MsgGutterSide::Top, 2, 3, 10), 0..0);
}

#[test]
fn bottom_gutter_screen_rows_stay_within_visible_content() {
    use tersse::pure::terminal_bounds::{content_max_y, row_is_visible};

    for height in 1..=5 {
        let rows = gutter_screen_rows(MsgGutterSide::Bottom, height, 23);
        assert_eq!(rows.end - rows.start, height as i32);
        assert_eq!(rows.end - 1, content_max_y(23));
        for screen_y in rows {
            assert!(row_is_visible(screen_y, 23));
        }
    }
}

#[test]
fn row_above_bottom_gutter_must_clip_printable_columns() {
    let gutter_rows = gutter_screen_rows(MsgGutterSide::Bottom, 1, 23);
    assert!(row_printing_wraps_into_gutter_block(
        gutter_rows.clone(),
        21
    ));
    assert!(!row_printing_wraps_into_gutter_block(gutter_rows, 20));

    let full_width_cols = 80;
    assert_eq!(
        clip_cols_to_avoid_wrapping_into_row(full_width_cols, 0, 79, true),
        79
    );
    assert_eq!(
        clip_cols_to_avoid_wrapping_into_row(full_width_cols, 0, 79, false),
        full_width_cols
    );
}

#[test]
fn element_intersection_uses_screen_scroll_offset() {
    let rows = 0..1;
    assert!(element_row_intersects_gutter_screen_rows(
        0,
        1,
        0,
        0,
        rows.clone()
    ));
    assert!(!element_row_intersects_gutter_screen_rows(
        2,
        1,
        0,
        0,
        rows.clone()
    ));
    assert!(element_row_intersects_gutter_screen_rows(3, 1, 3, 0, rows));
}

#[test]
fn viewport_height_shrinks_when_gutter_visible() {
    assert_eq!(viewport_height_for_screen_scroll(24, true, 3, MsgGutterSide::Bottom), 21);
    assert_eq!(viewport_height_for_screen_scroll(24, true, 0, MsgGutterSide::Bottom), 24);
    assert_eq!(viewport_height_for_screen_scroll(24, true, 3, MsgGutterSide::Top), 21);
    assert_eq!(viewport_height_for_screen_scroll(24, false, 3, MsgGutterSide::Bottom), 24);
}

#[test]
fn scroll_never_hides_message_gutter() {
    assert!(!should_hide_gutter_by_scroll_reveal(
        6, 0, 30, 24, 3, MsgGutterSide::Bottom
    ));
    assert!(!should_hide_gutter_by_scroll_reveal(
        9, 0, 30, 24, 3, MsgGutterSide::Bottom
    ));
    assert!(!should_hide_gutter_by_scroll_reveal(
        3, 0, 20, 24, 3, MsgGutterSide::Bottom
    ));
    assert!(!should_hide_gutter_by_scroll_reveal(
        0, 3, 30, 24, 3, MsgGutterSide::Top
    ));
    assert!(!should_hide_gutter_by_scroll_reveal(
        0, 99, 20, 24, 3, MsgGutterSide::Top
    ));
}

#[test]
fn max_screen_scroll_uses_effective_viewport_not_double_bonus() {
    let effective = viewport_height_for_screen_scroll(24, true, 3, MsgGutterSide::Bottom);
    assert_eq!(max_screen_scroll_offset(30, effective, None), 9);
    assert_eq!(max_screen_scroll_offset(30, 24, None), 6);
    assert_eq!(max_screen_scroll_offset(30, 24, Some(13)), 13);
    assert_eq!(max_screen_scroll_offset(30, 24, Some(3)), 6);
}

#[test]
fn scroll_down_with_gutter_stops_at_effective_viewport_max() {
    let effective = viewport_height_for_screen_scroll(24, true, 3, MsgGutterSide::Bottom);
    assert_eq!(scroll_screen_down_with_gutter(8, 30, effective, None), 9);
    assert_eq!(scroll_screen_down_with_gutter(9, 30, effective, None), 9);
}

#[test]
fn reveal_scroll_cap_ratchet_on_scroll_up() {
    let base_max = 6;
    assert_eq!(
        ratchet_gutter_scroll_cap_on_up(Some(13), 11, base_max),
        Some(11)
    );
    assert_eq!(
        ratchet_gutter_scroll_cap_on_up(Some(13), 13, base_max),
        Some(13)
    );
    assert_eq!(
        ratchet_gutter_scroll_cap_on_up(Some(13), 3, base_max),
        Some(6)
    );
    assert_eq!(
        ratchet_gutter_scroll_cap_on_up(Some(13), 6, base_max),
        Some(6)
    );
    assert_eq!(ratchet_gutter_scroll_cap_on_up(None, 5, base_max), None);
}

#[test]
fn top_reveal_scroll_cap_ratchet_on_scroll_down() {
    assert_eq!(
        ratchet_gutter_up_reveal_cap_on_down(Some(3), 2),
        Some(2)
    );
    assert_eq!(
        ratchet_gutter_up_reveal_cap_on_down(Some(3), 0),
        Some(0)
    );
    assert_eq!(ratchet_gutter_up_reveal_cap_on_down(None, 2), None);
}

#[test]
fn top_gutter_up_reveal_and_padding() {
    assert_eq!(max_screen_scroll_up_reveal(30, 24, true, 3, MsgGutterSide::Top, None), 3);
    assert_eq!(
        scroll_screen_up_with_gutter(2, 30, 24, 3, None),
        3
    );
    assert_eq!(
        clamp_screen_scroll_up_reveal_with_gutter(5, 30, 24, false, 3, MsgGutterSide::Top, Some(2)),
        2
    );
    assert!(screen_scroll_shows_top_padding(0, 3, false));
    assert!(!screen_scroll_shows_top_padding(0, 3, true));
    assert!(screen_scroll_shows_top_padding(2, 3, false));
    assert_eq!(top_padding_screen_rows(0, 3), 0..3);
    assert_eq!(top_padding_screen_rows(2, 3), 0..1);
    assert_eq!(top_padding_screen_rows(3, 3), 0..0);
}

#[test]
fn top_padding_after_hide_only_for_bonus_reveal_not_gutter_band() {
    assert_eq!(top_padding_screen_rows(0, 0), 0..0);
    assert_eq!(top_padding_screen_rows(0, 3), 0..3);
    assert_eq!(top_padding_screen_rows(2, 3), 0..1);
}

#[test]
fn clamp_screen_scroll_respects_reveal_cap() {
    assert_eq!(
        clamp_screen_scroll_with_gutter(20, 30, 24, Some(13)),
        13
    );
    let effective = viewport_height_for_screen_scroll(24, true, 3, MsgGutterSide::Bottom);
    assert_eq!(clamp_screen_scroll_with_gutter(13, 30, effective, None), 9);
    assert_eq!(
        clamp_screen_scroll_with_gutter(13, 30, 24, Some(13)),
        13
    );
    assert_eq!(clamp_screen_scroll_with_gutter(3, 30, 24, Some(3)), 3);
    assert_eq!(scroll_screen_down_with_gutter(5, 30, 24, Some(3)), 6);
    assert_eq!(scroll_screen_down_with_gutter(6, 30, 24, Some(3)), 6);
}

#[test]
fn padding_rows_not_drawn_while_gutter_visible() {
    assert!(!screen_scroll_shows_padding(9, 30, 24, true));
    assert!(screen_scroll_shows_padding(9, 30, 24, false));
    assert!(!screen_scroll_shows_padding(6, 30, 24, false));
    assert_eq!(padding_screen_rows(9, 30, 23), 21..23);
}
