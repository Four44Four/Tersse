use crate::clipboard;
use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input;
use crate::pure::text_wrap;
use crate::terminal_input::TerminalKey;

use super::types::UiEvent;
use super::RuntimeUi;

impl RuntimeUi {
    fn text_input_height_change(&mut self, id: crate::ElementId, before_text: &str) -> bool {
        let width = match self.element_by_id(id) {
            Some(element) if element.text_input.is_some() => element.width.max(1),
            _ => 1,
        };
        let before = super::layout::render_height_for_text_input_text(before_text, width);
        let after = self.text_input_render_height(id).unwrap_or(before);
        after != before
    }

    pub(super) fn handle_text_input_redraw_after_edit(
        &mut self,
        id: crate::ElementId,
        before_text: &str,
    ) {
        if self.text_input_height_change(id, before_text) {
            self.redraw_text_input_and_below(id);
        } else {
            self.redraw_keyboard_current_element(Some(id));
        }
    }

    pub(super) fn handle_display_scroll(&mut self, key: TerminalKey) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(element) = self.element_by_id(id) else {
            return false;
        };
        if element.text_input.is_some() {
            return false;
        }
        if !matches!(
            element.height_mode,
            super::types::ElementHeightMode::Fixed(_)
        ) {
            return false;
        }

        match key {
            TerminalKey::AltUp => {
                let Some(display) = self.element_mut_by_id(id) else {
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
                    let Some(display) = self.element_by_id(id) else {
                        return false;
                    };
                    let width = display.width.max(1);
                    let total = text_wrap::wrapped_line_count(&display.text, width);
                    let scroll = display.scroll;
                    let (max_y, max_x) = self.win.get_max_yx();
                    let height = match display.height_mode {
                        super::types::ElementHeightMode::Fixed(height) => height.max(1),
                        super::types::ElementHeightMode::FitContent => 1,
                    };
                    let (_, viewport_h) = terminal_bounds::clip_rect(
                        display.location.x as i32,
                        self.scrolled_y(display.location.y as i32),
                        width as i32,
                        height as i32,
                        max_x,
                        max_y,
                    );
                    (total, scroll, viewport_h.max(1) as usize)
                };
                let Some(display) = self.element_mut_by_id(id) else {
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
        let Some(input) = self.element_by_id(id) else {
            return false;
        };
        let Some(text_input) = input.text_input.as_ref() else {
            return false;
        };
        if text_input.locked {
            return false;
        }
        let before_text = input.text.clone();
        let state = self.text_input_state(id);
        if let Some(pasted) = text_input::paste_text(&state, paste) {
            self.apply_text_input_paste(id, pasted);
            self.handle_text_input_redraw_after_edit(id, &before_text);
        }
        true
    }

    pub(super) fn handle_text_input_editing(&mut self, key: TerminalKey) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(input) = self.element_by_id(id) else {
            return false;
        };
        let Some(text_input_behavior) = input.text_input.as_ref() else {
            return false;
        };
        let before_text = input.text.clone();

        let locked = text_input_behavior.locked;
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

        let state = self.text_input_state(id);
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
                        self.set_text_input_state(id, updated);
                        self.handle_text_input_redraw_after_edit(id, &before_text);
                    }
                }
                return true;
            }
            TerminalKey::Cut => {
                if let Some((updated, cut)) = text_input::cut_selection(&state) {
                    if clipboard::set_text(&cut) {
                        self.set_text_input_state(id, updated);
                        self.handle_text_input_redraw_after_edit(id, &before_text);
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

        self.apply_text_input_state(id, next_state);
        self.handle_text_input_redraw_after_edit(id, &before_text);
        true
    }

    pub(super) fn activate_button_on_focus(&mut self) -> UiEvent {
        let Some(id) = self.current_focused_id() else {
            return UiEvent::None;
        };
        let mut callback = {
            let Some(element) = self.element_mut_by_id(id) else {
                return UiEvent::None;
            };
            element.on_activate.take()
        };
        if let Some(handler) = callback.as_mut() {
            handler(self);
        }
        if let Some(handler) = callback {
            if let Some(element) = self.element_mut_by_id(id) {
                if element.on_activate.is_none() {
                    element.on_activate = Some(handler);
                }
            }
        }
        UiEvent::None
    }
}
