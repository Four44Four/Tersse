//! Crossterm raw-mode keyboard input (cross-platform, including Shift+arrow modifiers).

use crate::pure::keyboard::arrow_extend_selection;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::io::{self, stdout};
use std::time::Duration;

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

/// Poll for a key press or terminal resize. Returns `Ok(None)` on timeout.
pub fn poll_terminal(timeout: Duration) -> io::Result<Option<TerminalPoll>> {
    let poll_timeout = timeout;
    loop {
        if !event::poll(poll_timeout)? {
            return Ok(None);
        }
        match event::read()? {
            Event::Key(key) => {
                return Ok(map_key_event(key).map(TerminalPoll::Key));
            }
            Event::Paste(paste) => return Ok(Some(TerminalPoll::Paste(paste))),
            Event::Resize(cols, rows) => {
                return Ok(Some(TerminalPoll::Resized { cols, rows }));
            }
            _ => continue,
        }
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
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => Some(TerminalKey::ShiftDown),
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
