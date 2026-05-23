use crate::clipboard;
use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input;
use crate::pure::text_wrap;
use crate::terminal_input::TerminalKey;

use super::types::{RuntimeElement, UiEvent};
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn handle_display_scroll(&mut self, key: TerminalKey) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(idx) = self.idx_of(&id) else {
            return false;
        };
        let Some(RuntimeElement::TextDisplay(_)) = self.elements.get(idx) else {
            return false;
        };

        match key {
            TerminalKey::AltUp => {
                let Some(RuntimeElement::TextDisplay(display)) = self.elements.get_mut(idx) else {
                    return false;
                };
                if display.scroll == 0 {
                    return false;
                }
                display.scroll = scroll_view::scroll_line_up(display.scroll);
                true
            }
            TerminalKey::AltDown => {
                let (total, scroll, viewport_rows) = {
                    let RuntimeElement::TextDisplay(display) = &self.elements[idx] else {
                        return false;
                    };
                    let width = display.width.max(1);
                    let total = text_wrap::wrapped_line_count(&display.display.text, width);
                    let scroll = display.scroll;
                    let (max_y, max_x) = self.win.get_max_yx();
                    let (_, viewport_h) = terminal_bounds::clip_rect(
                        display.location.x as i32,
                        self.scrolled_y(display.location.y as i32),
                        width as i32,
                        display.height.max(1) as i32,
                        max_x,
                        max_y,
                    );
                    (total, scroll, viewport_h.max(1) as usize)
                };
                let Some(RuntimeElement::TextDisplay(display)) = self.elements.get_mut(idx) else {
                    return false;
                };
                display.scroll = scroll_view::scroll_line_down(scroll, total, viewport_rows);
                true
            }
            _ => false,
        }
    }

    pub(super) fn handle_text_input_paste(&mut self, paste: &str) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(RuntimeElement::TextInput(input)) = self.element_by_id(&id) else {
            return false;
        };
        if input.field.locked {
            return false;
        }
        let state = self.text_input_state(&id);
        if let Some(pasted) = text_input::paste_text(&state, paste) {
            self.apply_text_input_paste(&id, pasted);
        }
        true
    }

    pub(super) fn handle_text_input_editing(&mut self, key: TerminalKey) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(RuntimeElement::TextInput(input)) = self.element_by_id(&id) else {
            return false;
        };

        let locked = input.field.locked;
        if matches!(key, TerminalKey::Up | TerminalKey::Down) {
            match key {
                TerminalKey::Up => self.focus_prev(),
                TerminalKey::Down => self.focus_next(),
                _ => {}
            }
            return true;
        }

        if locked {
            return false;
        }

        let state = self.text_input_state(&id);
        let next_state = match key {
            TerminalKey::Left { extend_selection } => {
                text_input::cursor_left(&state, extend_selection)
            }
            TerminalKey::Right { extend_selection } => {
                text_input::cursor_right(&state, extend_selection)
            }
            TerminalKey::Backspace => text_input::backspace(&state),
            TerminalKey::Delete => text_input::delete_forward(&state),
            TerminalKey::Enter => text_input::insert_newline(&state),
            TerminalKey::Space => text_input::insert_char(&state, ' '),
            TerminalKey::Tab => text_input::insert_tab(&state),
            TerminalKey::Copy => {
                if let Some((updated, copied)) = text_input::copy_selection(&state) {
                    if clipboard::set_text(&copied) {
                        self.set_text_input_state(&id, updated);
                    }
                }
                return true;
            }
            TerminalKey::Cut => {
                if let Some((updated, cut)) = text_input::cut_selection(&state) {
                    if clipboard::set_text(&cut) {
                        self.set_text_input_state(&id, updated);
                    }
                }
                return true;
            }
            TerminalKey::Paste => {
                if let Some(paste) = clipboard::get_text() {
                    return self.handle_text_input_paste(&paste);
                }
                return true;
            }
            TerminalKey::Quit => text_input::insert_char(&state, 'q'),
            TerminalKey::Char(c) if c == '\t' => text_input::insert_tab(&state),
            TerminalKey::Char(c) if !c.is_control() => text_input::insert_char(&state, c),
            _ => return false,
        };

        self.apply_text_input_state(&id, next_state);
        true
    }

    pub(super) fn activate_button_on_focus(&mut self) -> UiEvent {
        let Some(id) = self.current_focused_id() else {
            return UiEvent::None;
        };
        let mut callback = {
            let Some(RuntimeElement::Button(button)) = self.element_mut_by_id(&id) else {
                return UiEvent::None;
            };
            button.on_press.take()
        };
        if let Some(handler) = callback.as_mut() {
            handler(self);
        }
        if let Some(handler) = callback {
            if let Some(RuntimeElement::Button(button)) = self.element_mut_by_id(&id) {
                if button.on_press.is_none() {
                    button.on_press = Some(handler);
                }
            }
        }
        UiEvent::None
    }
}
