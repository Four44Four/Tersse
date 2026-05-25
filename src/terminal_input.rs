//! Crossterm raw-mode keyboard input (cross-platform, including Shift+arrow modifiers).

use crate::constants::TERMINAL_POLL_COALESCE_IDLE_MS;
use crate::pure::keyboard::arrow_extend_selection;
use std::time::Duration;
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, EventStream, KeyCode, KeyEvent,
    KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use futures_util::StreamExt;
use std::io::{self, stdout};

/// Normalized key events for the UI loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalKey {
    Tab,
    /// Escape key — always exits the app.
    Escape,
    /// `q` / `Q` — exits when focus is not on the AI input field.
    Quit,
    Enter,
    Space,
    Backspace,
    Delete,
    Left {
        extend_selection: bool,
    },
    Right {
        extend_selection: bool,
    },
    Up,
    Down,
    ShiftUp,
    ShiftDown,
    AltUp,
    AltDown,
    /// Ctrl+C — copy selection (text input fields only).
    Copy,
    /// Ctrl+X — cut selection (text input fields only).
    Cut,
    /// Ctrl+V — paste from clipboard (text input fields only).
    Paste,
    Char(char),
}

/// Result of polling the terminal for input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalPoll {
    Key(TerminalKey),
    /// Full paste from the terminal (bracketed paste mode).
    Paste(String),
    Resized {
        cols: u16,
        rows: u16,
    },
}

pub fn enter_raw_mode() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnableBracketedPaste)?;
    Ok(())
}

pub fn leave_raw_mode() -> io::Result<()> {
    let _ = execute!(stdout(), DisableBracketedPaste);
    disable_raw_mode()
}

/// Current terminal size as `(columns, rows)`.
pub fn terminal_size() -> io::Result<(u16, u16)> {
    size()
}

pub fn terminal_event_stream() -> EventStream {
    EventStream::new()
}

/// Reads the next mapped terminal event from `EventStream`.
///
/// Returns `Ok(None)` when the stream ends.
pub async fn read_terminal_event(stream: &mut EventStream) -> io::Result<Option<TerminalPoll>> {
    while let Some(event) = stream.next().await {
        if let Some(poll) = map_crossterm_event(event.map_err(io::Error::other)?)? {
            return Ok(Some(poll));
        }
    }
    Ok(None)
}

/// Reads one blocking event, then drains any immediately available events and coalesces text runs.
///
/// Terminals without bracketed paste (notably Windows WinAPI input) deliver pasted text as a
/// burst of key events; merging them here yields a single [`TerminalPoll::Paste`] per burst.
pub async fn read_terminal_poll_batch(stream: &mut EventStream) -> io::Result<Option<Vec<TerminalPoll>>> {
    let Some(first) = read_terminal_event(stream).await? else {
        return Ok(None);
    };
    let mut batch = vec![first];
    let idle = Duration::from_millis(TERMINAL_POLL_COALESCE_IDLE_MS);
    loop {
        match tokio::time::timeout(idle, read_terminal_event(stream)).await {
            Ok(Ok(Some(poll))) => batch.push(poll),
            Ok(Ok(None)) => break,
            Ok(Err(err)) => return Err(err),
            Err(_) => break,
        }
    }
    Ok(Some(coalesce_terminal_poll_batch(batch)))
}

pub(crate) fn coalesce_terminal_poll_batch(batch: Vec<TerminalPoll>) -> Vec<TerminalPoll> {
    let mut out = Vec::new();
    let mut text = String::new();

    let flush = |out: &mut Vec<TerminalPoll>, text: &mut String| {
        if !text.is_empty() {
            out.push(TerminalPoll::Paste(std::mem::take(text)));
        }
    };

    for poll in batch {
        match poll {
            TerminalPoll::Paste(s) => text.push_str(&s),
            TerminalPoll::Key(TerminalKey::Char(c)) if coalesce_text_char(c) => text.push(c),
            // Space/Enter/Tab mid-paste are part of a Windows paste burst; alone they are key presses.
            TerminalPoll::Key(TerminalKey::Tab) if !text.is_empty() => text.push('\t'),
            TerminalPoll::Key(TerminalKey::Space) if !text.is_empty() => text.push(' '),
            TerminalPoll::Key(TerminalKey::Enter) if !text.is_empty() => text.push('\n'),
            other => {
                flush(&mut out, &mut text);
                out.push(other);
            }
        }
    }
    flush(&mut out, &mut text);
    out
}

fn coalesce_text_char(c: char) -> bool {
    !c.is_control() || c == '\t'
}

fn map_crossterm_event(event: Event) -> io::Result<Option<TerminalPoll>> {
    Ok(match event {
        Event::Key(key) => map_key_event(key).map(TerminalPoll::Key),
        Event::Paste(paste) => Some(TerminalPoll::Paste(paste)),
        Event::Resize(cols, rows) => Some(TerminalPoll::Resized { cols, rows }),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesce_char_burst_into_single_paste() {
        let batch = vec![
            TerminalPoll::Key(TerminalKey::Char('a')),
            TerminalPoll::Key(TerminalKey::Char('b')),
            TerminalPoll::Key(TerminalKey::Char('c')),
        ];
        assert_eq!(
            coalesce_terminal_poll_batch(batch),
            vec![TerminalPoll::Paste("abc".to_string())]
        );
    }

    #[test]
    fn coalesce_preserves_non_text_keys() {
        let batch = vec![
            TerminalPoll::Key(TerminalKey::Char('a')),
            TerminalPoll::Key(TerminalKey::Left {
                extend_selection: false,
            }),
        ];
        assert_eq!(
            coalesce_terminal_poll_batch(batch),
            vec![
                TerminalPoll::Paste("a".to_string()),
                TerminalPoll::Key(TerminalKey::Left {
                    extend_selection: false,
                }),
            ]
        );
    }

    #[test]
    fn coalesce_keeps_standalone_space_and_enter_as_keys() {
        assert_eq!(
            coalesce_terminal_poll_batch(vec![TerminalPoll::Key(TerminalKey::Space)]),
            vec![TerminalPoll::Key(TerminalKey::Space)]
        );
        assert_eq!(
            coalesce_terminal_poll_batch(vec![TerminalPoll::Key(TerminalKey::Enter)]),
            vec![TerminalPoll::Key(TerminalKey::Enter)]
        );
    }

    #[test]
    fn coalesce_enter_after_chars_as_paste_newline() {
        let batch = vec![
            TerminalPoll::Key(TerminalKey::Char('a')),
            TerminalPoll::Key(TerminalKey::Enter),
            TerminalPoll::Key(TerminalKey::Char('b')),
        ];
        assert_eq!(
            coalesce_terminal_poll_batch(batch),
            vec![TerminalPoll::Paste("a\nb".to_string())]
        );
    }
}

fn map_key_event(key: KeyEvent) -> Option<TerminalKey> {
    if key.kind == KeyEventKind::Release {
        return None;
    }

    let extend = arrow_extend_selection(key.modifiers.contains(KeyModifiers::SHIFT));

    match key.code {
        KeyCode::Tab => Some(TerminalKey::Tab),
        KeyCode::Esc => Some(TerminalKey::Escape),
        KeyCode::Enter => Some(TerminalKey::Enter),
        KeyCode::Backspace => Some(TerminalKey::Backspace),
        KeyCode::Delete => Some(TerminalKey::Delete),
        KeyCode::Left => Some(TerminalKey::Left {
            extend_selection: extend,
        }),
        KeyCode::Right => Some(TerminalKey::Right {
            extend_selection: extend,
        }),
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => Some(TerminalKey::ShiftUp),
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
            Some(TerminalKey::ShiftDown)
        }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => Some(TerminalKey::AltUp),
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => Some(TerminalKey::AltDown),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(TerminalKey::Copy)
        }
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(TerminalKey::Cut)
        }
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(TerminalKey::Paste)
        }
        KeyCode::Up => Some(TerminalKey::Up),
        KeyCode::Down => Some(TerminalKey::Down),
        KeyCode::Char('q' | 'Q') => Some(TerminalKey::Quit),
        KeyCode::Char(' ') => Some(TerminalKey::Space),
        KeyCode::Char(c) if c == '\x08' || c == '\x7f' => Some(TerminalKey::Backspace),
        KeyCode::Char(c) => Some(TerminalKey::Char(c)),
        _ => None,
    }
}
