use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input::{self, TextInputState};
use crate::pure::text_wrap;
use crate::TitleAlignment;
use pancurses::{curs_set, COLOR_PAIR};

use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
    pub fn draw(&mut self) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();
        self.win.erase();

        if let Some(title) = &self.title {
            self.draw_title(title.clone());
        }

        for idx in 0..self.elements.len() {
            match &self.elements[idx] {
                RuntimeElement::Button(_) => self.draw_button(idx),
                RuntimeElement::TextInput(_) => self.draw_text_input(idx),
                RuntimeElement::TextDisplay(_) => self.draw_text_display(idx),
            }
        }

        self.draw_cursor_for_active_text_input();
        self.win.refresh();
    }

    fn draw_title(&mut self, title: crate::ScreenTitle) {
        let pair = self.color_pair(title.fg_color, title.bg_color);
        let max_x = self.win.get_max_x().max(1);
        let text_len = title.text.chars().count() as i32;
        let col = match title.alignment {
            TitleAlignment::Left => 0,
            TitleAlignment::Right => (max_x - text_len).max(0),
            TitleAlignment::Center => ((max_x - text_len) / 2).max(0),
        };
        self.win.attron(COLOR_PAIR(pair as u64));
        let title_y = self.scrolled_y(0);
        if !terminal_bounds::row_is_visible(title_y, self.win.get_max_y()) {
            return;
        }
        self.win.mv(title_y, col);
        self.win.addstr(&title.text);
        self.win.attroff(COLOR_PAIR(pair as u64));
    }

    fn draw_button(&mut self, idx: usize) {
        let (location, text, width, style) = match &self.elements[idx] {
            RuntimeElement::Button(button) => {
                let style = if button.button.focused {
                    button.style.focused
                } else {
                    button.style.unfocused
                };
                (
                    button.button.location,
                    button.button.display_string.clone(),
                    button.button.width,
                    style,
                )
            }
            _ => return,
        };

        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        if !terminal_bounds::row_is_visible(y, max_y) {
            return;
        }
        let (_, draw_h) =
            terminal_bounds::clip_rect(x, y, width.max(1) as i32, 1, max_x, max_y);
        if draw_h <= 0 {
            return;
        }
        let row_cols = terminal_bounds::cols_for_printing(x, max_x, y, max_y) as usize;
        let draw_width = width.max(1).min(row_cols);

        let label = crate::pure::button::truncate_label(&text, draw_width);
        let pad_cols = crate::pure::button::padding_cols(&label, draw_width);

        let pair = self.color_pair(style.fg, style.bg);
        self.win.attron(COLOR_PAIR(pair as u64));
        self.win.mv(y, x);
        if !label.is_empty() {
            self.win.addstr(&label);
        }
        for _ in 0..pad_cols {
            self.win.addch(' ');
        }
        self.win.attroff(COLOR_PAIR(pair as u64));
    }

    fn draw_text_input(&mut self, idx: usize) {
        let (location, width, text, cursor, selection_anchor, style) = match &self.elements[idx] {
            RuntimeElement::TextInput(input) => {
                let style = if input.field.locked {
                    if input.field.focused {
                        input.style.focused_locked
                    } else {
                        input.style.unfocused_locked
                    }
                } else if input.field.focused {
                    input.style.focused_unlocked
                } else {
                    input.style.unfocused_unlocked
                };
                (
                    input.location,
                    input.field.width.max(1),
                    input.field.text.clone(),
                    input.cursor,
                    input.selection_anchor,
                    (style, input.style.selection),
                )
            }
            _ => return,
        };

        let base_pair = self.color_pair(style.0.fg, style.0.bg);
        let selection_pair = self.color_pair(style.1.fg, style.1.bg);
        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        let lines = text_wrap::wrapped_lines(&text, width);
        let logical_rows = text_wrap::display_row_count(&text, width) as i32;
        let draw_w = terminal_bounds::clip_rect(x, y, width as i32, 1, max_x, max_y).0;
        if draw_w <= 0 || logical_rows <= 0 {
            return;
        }

        self.fill_solid_rows(y, x, draw_w, logical_rows, base_pair);

        let state = TextInputState {
            text: text.clone(),
            cursor,
            selection_anchor,
        };
        let selection = text_input::selection_range(&state);
        let highlight_cells = text_wrap::selection_highlight_cells(&text, selection, width);

        let visible_lines =
            terminal_bounds::visible_element_line_range(y, logical_rows, max_y);
        let visible_start = visible_lines.start;
        let visible_end = visible_lines.end;
        for line_idx in visible_lines {
            let line_idx = line_idx as usize;
            let row_y = y + line_idx as i32;
            let max_cols = terminal_bounds::max_element_row_cols(
                x,
                max_x,
                row_y,
                max_y,
                width as i32,
            ) as usize;
            if max_cols == 0 {
                continue;
            }
            let line = lines.get(line_idx).map(String::as_str).unwrap_or("");
            for (col, ch) in line.chars().enumerate() {
                if col >= max_cols {
                    break;
                }
                let pair = if highlight_cells.contains(&(line_idx, col)) {
                    selection_pair
                } else {
                    base_pair
                };
                self.win.attron(COLOR_PAIR(pair as u64));
                self.win.mv(row_y, x + col as i32);
                self.win.addch(ch);
                self.win.attroff(COLOR_PAIR(pair as u64));
            }
            for col in line.chars().count()..max_cols {
                if highlight_cells.contains(&(line_idx, col)) {
                    self.win.attron(COLOR_PAIR(selection_pair as u64));
                    self.win.mv(row_y, x + col as i32);
                    self.win.addch(' ');
                    self.win.attroff(COLOR_PAIR(selection_pair as u64));
                }
            }
        }

        for (line_idx, col) in highlight_cells {
            let line_idx = line_idx as i32;
            if line_idx < visible_start || line_idx >= visible_end {
                continue;
            }
            let row_y = y + line_idx;
            let max_cols = terminal_bounds::max_element_row_cols(
                x,
                max_x,
                row_y,
                max_y,
                width as i32,
            ) as usize;
            if col >= max_cols {
                continue;
            }
            let line_len = lines
                .get(line_idx as usize)
                .map(|line| line.chars().count())
                .unwrap_or(0);
            if col < line_len {
                continue;
            }
            self.win.attron(COLOR_PAIR(selection_pair as u64));
            self.win.mv(row_y, x + col as i32);
            self.win.addch(' ');
            self.win.attroff(COLOR_PAIR(selection_pair as u64));
        }
    }

    fn draw_text_display(&mut self, idx: usize) {
        let (location, width, height, text, scroll, style) = match &self.elements[idx] {
            RuntimeElement::TextDisplay(display) => {
                let style = if display.display.focused {
                    display.style.focused
                } else {
                    display.style.unfocused
                };
                (
                    display.location,
                    display.width.max(1),
                    display.height.max(1),
                    display.display.text.clone(),
                    display.scroll,
                    style,
                )
            }
            _ => return,
        };

        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        let (draw_w, draw_h) =
            terminal_bounds::clip_rect(x, y, width as i32, height as i32, max_x, max_y);
        if draw_w <= 0 || draw_h <= 0 {
            return;
        }
        let draw_rows = draw_h as usize;

        let pair = self.color_pair(style.fg, style.bg);
        self.fill_solid(y, x, draw_w, draw_h, pair);

        let lines = text_wrap::wrapped_lines(&text, width);
        if lines.is_empty() {
            return;
        }
        let offset = scroll_view::clamp_scroll_offset(scroll, lines.len(), draw_rows);
        let range = scroll_view::visible_line_range(offset, draw_rows, lines.len());

        self.win.attron(COLOR_PAIR(pair as u64));
        for (row, line_idx) in range.enumerate() {
            let row_y = y + row as i32;
            if !terminal_bounds::row_is_visible(row_y, max_y) {
                continue;
            }
            let row_cols = terminal_bounds::max_element_row_cols(
                x,
                max_x,
                row_y,
                max_y,
                width as i32,
            ) as usize;
            self.win.mv(row_y, x);
            let line = terminal_bounds::clip_str_to_cols(&lines[line_idx], row_cols);
            self.win.addstr(&line);
        }
        self.win.attroff(COLOR_PAIR(pair as u64));
    }

    fn fill_solid(&self, y: i32, x: i32, w: i32, h: i32, pair: i16) {
        let (max_y, max_x) = self.win.get_max_yx();
        let (w, h) = terminal_bounds::clip_rect(x, y, w, h, max_x, max_y);
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_solid_rows(y, x, w, h, pair);
    }

    /// Fills a solid background across `logical_rows` rows, clipping width to the terminal
    /// and skipping rows that are off-screen (including above the viewport when scrolled).
    fn fill_solid_rows(&self, y: i32, x: i32, w: i32, logical_rows: i32, pair: i16) {
        let (max_y, max_x) = self.win.get_max_yx();
        let w = w.min(terminal_bounds::cols_visible_from(x, max_x)).max(0);
        if w <= 0 || logical_rows <= 0 {
            return;
        }
        let visible_rows =
            terminal_bounds::visible_element_line_range(y, logical_rows, max_y);
        self.win.attron(COLOR_PAIR(pair as u64));
        for row in visible_rows {
            let row_y = y + row;
            let row_w = terminal_bounds::cols_for_printing(x, max_x, row_y, max_y).min(w);
            if row_w <= 0 {
                continue;
            }
            self.win.mv(row_y, x);
            for _ in 0..row_w {
                self.win.addch(' ');
            }
        }
        self.win.attroff(COLOR_PAIR(pair as u64));
    }

    fn draw_cursor_for_active_text_input(&mut self) {
        let Some(focused_id) = self.current_focused_id() else {
            let _ = curs_set(0);
            return;
        };

        let Some(RuntimeElement::TextInput(input)) = self.element_by_id(&focused_id) else {
            let _ = curs_set(0);
            return;
        };

        if input.field.locked {
            let _ = curs_set(0);
            return;
        }

        let width = input.field.width.max(1);
        let (line, col) =
            text_wrap::cursor_display_position(&input.field.text, input.cursor, width);
        let (max_y, max_x) = self.win.get_max_yx();
        let x = input.location.x as i32;
        let y = self.scrolled_y(input.location.y as i32);
        let row_y = y + line as i32;
        if !terminal_bounds::row_is_visible(row_y, max_y) {
            let _ = curs_set(0);
            return;
        }
        let max_cols = terminal_bounds::max_element_row_cols(x, max_x, row_y, max_y, width as i32);
        if col as i32 >= max_cols {
            let _ = curs_set(0);
            return;
        }
        let _ = curs_set(1);
        self.win.mv(row_y, x + col as i32);
    }
}
