use std::time::Duration;

use pancurses::{curs_set, endwin, initscr, noecho};

use crate::terminal_input;
use crate::terminal_input::{TerminalKey, TerminalPoll};
use crate::ScreenTitle;

use super::types::UiEvent;
use super::RuntimeUi;

impl RuntimeUi {
    pub fn new() -> Self {
        let _ = terminal_input::enter_raw_mode();
        let win = initscr();
        noecho();
        let _ = curs_set(0);
        pancurses::start_color();
        pancurses::use_default_colors();

        let mut ui = Self {
            win,
            title: None,
            elements: Vec::new(),
            focused_position: 0,
            pair_cache: std::collections::HashMap::new(),
            next_pair_id: 1,
            cached_heights: std::collections::HashMap::new(),
            text_input_layout_cache: std::collections::HashMap::new(),
            resize_debounce_until: None,
            last_terminal_yx: None,
            screen_scroll: 0,
        };
        let _ = ui.reload_screen_after_resize();
        ui
    }

    pub fn set_title(&mut self, title: ScreenTitle) {
        self.title = Some(title);
    }

    pub fn clear_title(&mut self) {
        self.title = None;
    }

    /// Draw one frame and process one input event.
    ///
    /// Returns `false` when the runtime receives a quit key.
    pub fn run_frame(&mut self, timeout: Duration) -> bool {
        let quit = matches!(self.poll_event(timeout), UiEvent::Quit);
        let _ = self.tick_resize_debounce();
        if !self.is_resize_debounce_active() {
            self.draw();
        }
        !quit
    }

    pub fn poll_event(&mut self, timeout: Duration) -> UiEvent {
        match terminal_input::poll_terminal(timeout) {
            Ok(Some(TerminalPoll::Resized { .. })) => {
                self.note_terminal_resize();
                UiEvent::None
            }
            Ok(Some(TerminalPoll::Key(key))) => self.handle_key(key),
            Ok(None) => UiEvent::None,
            Err(_) => UiEvent::Quit,
        }
    }

    fn handle_key(&mut self, key: TerminalKey) -> UiEvent {
        if self.handle_screen_scroll(key) {
            return UiEvent::None;
        }

        if self.handle_display_scroll(key) {
            return UiEvent::None;
        }

        if self.handle_text_input_editing(key) {
            return UiEvent::None;
        }

        match key {
            TerminalKey::Quit | TerminalKey::Escape => UiEvent::Quit,
            TerminalKey::Up | TerminalKey::Left { .. } => {
                self.focus_prev();
                UiEvent::None
            }
            TerminalKey::Down | TerminalKey::Right { .. } => {
                self.focus_next();
                UiEvent::None
            }
            TerminalKey::Enter | TerminalKey::Space => self.activate_button_on_focus(),
            _ => UiEvent::None,
        }
    }
}

impl Drop for RuntimeUi {
    fn drop(&mut self) {
        let _ = curs_set(1);
        endwin();
        let _ = terminal_input::leave_raw_mode();
    }
}
