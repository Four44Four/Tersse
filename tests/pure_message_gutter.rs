use tersse::pure::message_gutter::{
    apply_message, element_row_intersects_gutter_screen_rows, gutter_rows_to_restore,
    gutter_screen_rows, hide_message, layout_message_gutter_lines, message_gutter_height,
    title_intersects_gutter_screen_rows, MessageGutterState, MsgGutterSide,
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
    assert_eq!(
        gutter_rows_to_restore(MsgGutterSide::Top, 3, 1, 10),
        1..3
    );
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
fn element_intersection_uses_screen_scroll_offset() {
    let rows = 0..1;
    assert!(element_row_intersects_gutter_screen_rows(0, 1, 0, rows.clone()));
    assert!(!element_row_intersects_gutter_screen_rows(2, 1, 0, rows.clone()));
    assert!(element_row_intersects_gutter_screen_rows(3, 1, 3, rows));
}

#[test]
fn title_intersection_accounts_for_screen_scroll() {
    assert!(title_intersects_gutter_screen_rows(true, 0, 0..1));
    assert!(!title_intersects_gutter_screen_rows(true, 1, 0..1));
}
