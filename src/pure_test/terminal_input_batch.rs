//! Pure model of terminal poll batch handling for coalesced paste + redraw semantics.
//!
//! Mirrors `RuntimeUi::handle_terminal_poll_batch` pending-text and redraw rules.

pub use crate::pure::terminal_input_batch::PendingPasteBuffer;
use crate::pure_test::terminal_poll_coalesce::{self, TerminalPollItem};

/// Poll shape used by the batch planner (resize/key/paste only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchPoll {
    Paste(String),
    /// A key handled on the `handle_key` path after any pending paste is flushed.
    TextKey {
        /// Whether handling the key runs `commit_text_input_redraw` (e.g. text-input edit).
        commits_text_input_redraw: bool,
        /// Whether handling the key sets `full_immediate` (e.g. screen scroll).
        full_immediate: bool,
    },
    Resize,
}

/// Counts of redraw-related effects produced while processing one batch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PasteRedrawSemantics {
    /// Calls to `handle_text_input_terminal_paste` (one per flushed pending buffer).
    pub apply_paste_count: usize,
    /// Calls to `redraw_keyboard_focused_elements` inside the batch loop.
    pub keyboard_focus_redraw_count: usize,
    /// Calls to `finish_terminal_input_redraw` after the loop.
    pub finish_terminal_input_redraw_count: usize,
}

/// Merges additional terminal batches from the UI signal queue (see `recv_terminal_poll_batch`).
pub fn merge_poll_batches<T>(first: Vec<T>, additional: impl IntoIterator<Item = Vec<T>>) -> Vec<T> {
    let mut batch = first;
    for more in additional {
        batch.extend(more);
    }
    batch
}

/// Plans paste apply + redraw effects using the same control flow as `handle_terminal_poll_batch`.
pub fn analyze_batch_redraw_semantics(batch: &[BatchPoll]) -> PasteRedrawSemantics {
    let mut pending = PendingPasteBuffer::new();
    let mut committed = false;
    let mut full_immediate = false;
    let mut semantics = PasteRedrawSemantics::default();

    let flush_pending = |pending: &mut PendingPasteBuffer,
                         semantics: &mut PasteRedrawSemantics,
                         committed: &mut bool| {
        if pending.flush_before_boundary().is_some() {
            semantics.apply_paste_count += 1;
            *committed = true;
        }
    };

    let has_keyboard = batch.iter().any(|poll| {
        matches!(poll, BatchPoll::Paste(_) | BatchPoll::TextKey { .. })
    });

    if has_keyboard {
        // `flush_pending_queue_redraw_for_keyboard` — not counted as text-input redraw.
    }

    for poll in batch {
        match poll {
            BatchPoll::Resize => {
                flush_pending(&mut pending, &mut semantics, &mut committed);
            }
            BatchPoll::Paste(paste) => pending.push_paste(&paste),
            BatchPoll::TextKey {
                commits_text_input_redraw,
                full_immediate: key_full_immediate,
            } => {
                flush_pending(&mut pending, &mut semantics, &mut committed);
                if !committed {
                    semantics.keyboard_focus_redraw_count += 1;
                }
                if *key_full_immediate {
                    full_immediate = true;
                }
                if *commits_text_input_redraw {
                    committed = true;
                }
            }
        }
    }

    flush_pending(&mut pending, &mut semantics, &mut committed);

    if !committed {
        semantics.finish_terminal_input_redraw_count += 1;
    } else if full_immediate {
        semantics.finish_terminal_input_redraw_count += 1;
    }

    semantics
}

/// True when a batch is paste-only and must produce exactly one paste apply and no extra redraws.
pub fn guarantees_single_coalesced_paste_redraw(semantics: &PasteRedrawSemantics) -> bool {
    semantics.apply_paste_count == 1
        && semantics.keyboard_focus_redraw_count == 0
        && semantics.finish_terminal_input_redraw_count == 0
}

/// True when the pending buffer yields a single flush for the given poll sequence.
/// End-to-end check: terminal coalesce → pending merge → single apply + single commit redraw.
pub fn coalesced_burst_guarantees_single_redraw(items: &[TerminalPollItem]) -> bool {
    let coalesced = terminal_poll_coalesce::coalesce_terminal_poll_items(items.to_vec());
    if coalesced.len() != 1 {
        return false;
    }
    let TerminalPollItem::Paste(text) = &coalesced[0] else {
        return false;
    };
    let batch = vec![BatchPoll::Paste(text.clone())];
    guarantees_single_coalesced_paste_redraw(&analyze_batch_redraw_semantics(&batch))
}

pub fn single_pending_paste_flush(batch: &[BatchPoll]) -> bool {
    let mut pending = PendingPasteBuffer::new();
    let mut flush_count = 0usize;

    let mut flush = |pending: &mut PendingPasteBuffer| {
        if pending.flush_before_boundary().is_some() {
            flush_count += 1;
        }
    };

    for poll in batch {
        match poll {
            BatchPoll::Resize => flush(&mut pending),
            BatchPoll::Paste(paste) => pending.push_paste(&paste),
            BatchPoll::TextKey { .. } => flush(&mut pending),
        }
    }
    flush(&mut pending);

    flush_count == 1
}
