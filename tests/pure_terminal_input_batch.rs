use tersse::pure_test::terminal_input_batch::{
    analyze_batch_redraw_semantics, coalesced_burst_guarantees_single_redraw,
    guarantees_single_coalesced_paste_redraw, merge_poll_batches, single_pending_paste_flush,
    BatchPoll, PendingPasteBuffer,
};
use tersse::pure_test::terminal_poll_coalesce::{
    coalesce_terminal_poll_items, TerminalPollItem,
};

fn paste_only_batch(chunks: &[&str]) -> Vec<BatchPoll> {
    chunks
        .iter()
        .map(|chunk| BatchPoll::Paste((*chunk).to_string()))
        .collect()
}

#[test]
fn pending_buffer_merges_multiple_paste_chunks_into_one_flush() {
    let mut pending = PendingPasteBuffer::new();
    pending.push_paste("hel");
    pending.push_paste("lo");
    assert!(pending.flush_end().is_some_and(|text| text == "hello"));
    assert!(pending.is_empty());
}

#[test]
fn pending_buffer_flushes_on_key_boundary_before_later_paste() {
    let mut pending = PendingPasteBuffer::new();
    pending.push_paste("a");
    assert_eq!(pending.flush_before_boundary().as_deref(), Some("a"));
    pending.push_paste("b");
    assert_eq!(pending.flush_end().as_deref(), Some("b"));
}

#[test]
fn single_bracketed_paste_poll_yields_one_apply_and_no_extra_redraws() {
    let batch = paste_only_batch(&["hello world"]);
    assert!(single_pending_paste_flush(&batch));

    let semantics = analyze_batch_redraw_semantics(&batch);
    assert_eq!(semantics.apply_paste_count, 1);
    assert!(guarantees_single_coalesced_paste_redraw(&semantics));
}

#[test]
fn multiple_paste_polls_in_one_batch_merge_to_one_apply() {
    let batch = paste_only_batch(&["abc", "def", "ghi"]);
    assert!(single_pending_paste_flush(&batch));

    let semantics = analyze_batch_redraw_semantics(&batch);
    assert_eq!(semantics.apply_paste_count, 1);
    assert!(guarantees_single_coalesced_paste_redraw(&semantics));
}

#[test]
fn coalesced_char_burst_plans_single_apply_and_redraw() {
    let items = vec![
        TerminalPollItem::Char('a'),
        TerminalPollItem::Char('b'),
        TerminalPollItem::Char('c'),
    ];
    assert!(coalesced_burst_guarantees_single_redraw(&items));
    let coalesced = coalesce_terminal_poll_items(items);
    assert_eq!(coalesced, vec![TerminalPollItem::Paste("abc".to_string())]);
}

#[test]
fn coalesced_paste_with_internal_whitespace_stays_one_apply() {
    let coalesced = coalesce_terminal_poll_items(vec![
        TerminalPollItem::Char('a'),
        TerminalPollItem::Space,
        TerminalPollItem::Tab,
        TerminalPollItem::Enter,
        TerminalPollItem::Char('b'),
    ]);
    assert_eq!(coalesced, vec![TerminalPollItem::Paste("a \t\nb".to_string())]);

    let batch = vec![BatchPoll::Paste("a \t\nb".to_string())];
    assert!(guarantees_single_coalesced_paste_redraw(&analyze_batch_redraw_semantics(
        &batch
    )));
}

#[test]
fn merged_signal_batches_still_yield_single_apply() {
    let merged = merge_poll_batches(
        vec![TerminalPollItem::Paste("foo".to_string())],
        [vec![TerminalPollItem::Paste("bar".to_string())]],
    );
    let coalesced = coalesce_terminal_poll_items(merged);
    assert_eq!(coalesced, vec![TerminalPollItem::Paste("foobar".to_string())]);

    let batch = vec![BatchPoll::Paste("foobar".to_string())];
    assert!(guarantees_single_coalesced_paste_redraw(&analyze_batch_redraw_semantics(
        &batch
    )));
}

#[test]
fn non_text_key_between_paste_chunks_splits_apply_but_not_mid_burst() {
    let batch = vec![
        BatchPoll::Paste("first".to_string()),
        BatchPoll::TextKey {
            commits_text_input_redraw: false,
            full_immediate: false,
        },
        BatchPoll::Paste("second".to_string()),
    ];
    assert!(!single_pending_paste_flush(&batch));

    let semantics = analyze_batch_redraw_semantics(&batch);
    assert_eq!(semantics.apply_paste_count, 2);
    // First paste flush already commits text-input redraw; key does not add focus redraw.
    assert_eq!(semantics.keyboard_focus_redraw_count, 0);
}

#[test]
fn text_key_edit_after_coalesced_paste_does_not_finish_redraw_twice() {
    let batch = vec![
        BatchPoll::Paste("pasted".to_string()),
        BatchPoll::TextKey {
            commits_text_input_redraw: true,
            full_immediate: false,
        },
    ];
    let semantics = analyze_batch_redraw_semantics(&batch);
    assert_eq!(semantics.apply_paste_count, 1);
    assert_eq!(semantics.keyboard_focus_redraw_count, 0);
    assert_eq!(semantics.finish_terminal_input_redraw_count, 0);
}
