//! Crossterm raw-mode keyboard input (cross-platform, including Shift+arrow modifiers).

use crate::pure::keyboard::arrow_extend_selection;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
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

pub fn enter_raw_mode() -> io::Result<()> {
    enable_raw_mode()
}

pub fn leave_raw_mode() -> io::Result<()> {
    disable_raw_mode()
}

/// Poll for a key press. Returns `Ok(None)` on timeout or non-key events.
pub fn poll_key(timeout: Duration) -> io::Result<Option<TerminalKey>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    Ok(match event::read()? {
        Event::Key(key) => map_key_event(key),
        _ => None,
    })
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
