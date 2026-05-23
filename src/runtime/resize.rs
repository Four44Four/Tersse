use std::time::{Duration, Instant};

use pancurses::resize_term;

use crate::constants::TERM_RESIZE_DEBOUNCE_MS;
use crate::pure::resize_debounce;
use crate::terminal_input;

use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn note_terminal_resize(&mut self) {
        self.resize_debounce_until = Some(resize_debounce::debounce_deadline(
            Instant::now(),
            Duration::from_millis(TERM_RESIZE_DEBOUNCE_MS),
        ));
    }

    pub(super) fn is_resize_debounce_active(&self) -> bool {
        self.resize_debounce_until.is_some_and(|until| {
            !resize_debounce::debounce_has_elapsed(until, Instant::now())
        })
    }

    /// Applies a debounced terminal resize. Returns true when the terminal size changed.
    pub(super) fn tick_resize_debounce(&mut self) -> bool {
        let Some(until) = self.resize_debounce_until else {
            return false;
        };
        if !resize_debounce::debounce_has_elapsed(until, Instant::now()) {
            return false;
        }
        self.resize_debounce_until = None;
        self.reload_screen_after_resize()
    }

    fn sync_curses_terminal_size(&mut self) {
        if let Ok((cols, rows)) = terminal_input::terminal_size() {
            if rows > 0 && cols > 0 {
                let _ = resize_term(rows as i32, cols as i32);
            }
        }
        let _ = resize_term(0, 0);
    }

    pub(super) fn reload_screen_after_resize(&mut self) -> bool {
        self.sync_curses_terminal_size();
        let (max_y, max_x) = self.win.get_max_yx();
        let changed = self.last_terminal_yx != Some((max_y, max_x));
        self.last_terminal_yx = Some((max_y, max_x));
        if changed {
            self.refresh_height_cache();
        }
        changed
    }
}
