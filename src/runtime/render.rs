use std::time::{Duration, Instant};

use crate::constants::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;
use crate::pure::focus_order;
use crate::pure::resize_debounce;
use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input::{self, TextInputState};
use crate::pure::text_wrap;
use crate::pure::ui_redraw;
use crate::ElementId;
use pancurses::{curs_set, COLOR_PAIR};

use super::types::ElementHeightMode;
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn request_draw(&mut self) {
        self.redraw_debounce_until.get_or_insert_with(|| {
            resize_debounce::debounce_deadline(
                Instant::now(),
                Duration::from_millis(UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS),
            )
        });
    }

    /// Applies a debounced UI redraw for queued UiSession updates.
    pub(super) fn tick_redraw_debounce(&mut self) -> bool {
        self.flush_pending_queue_redraw(false)
    }

    pub(super) fn next_debounce_deadline(&self) -> Option<Instant> {
        let queue_deadline = if self.ui_queue_redraw_pending {
            self.redraw_debounce_until
        } else {
            None
        };
        let primary = match (self.resize_debounce_until, queue_deadline) {
            (Some(resize), Some(redraw)) => Some(resize.min(redraw)),
            (Some(resize), None) => Some(resize),
            (None, Some(redraw)) => Some(redraw),
            (None, None) => None,
        };
        [primary, self.message_gutter_expiry_deadline()]
            .into_iter()
            .flatten()
            .min()
    }

    pub(super) fn flush_pending_redraw(&mut self) -> bool {
        let resize_changed = self.tick_resize_debounce();
        if self.is_resize_debounce_active() {
            return false;
        }
        if resize_changed {
            self.draw();
            self.clear_pending_queue_redraw();
            return true;
        }
        self.tick_redraw_debounce()
    }

    pub fn draw(&mut self) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();
        if self.message_gutter.visible {
            self.clear_screen_for_draw();
        } else {
            self.win.erase();
        }

        let draw_order: Vec<usize> = self.elements.focus_order_ids();
        for id in draw_order {
            self.draw_existing_element(id);
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        // Message gutter overlay is drawn only on apply/update (refresh) or hide; see
        // specifications/message_gutter.txt. clear_screen_for_draw already skips gutter rows.
        self.win.refresh();
        // A full-screen draw supersedes any incremental queue redraw plan (e.g. marks
        // accumulated while creating elements before the first frame).
        self.clear_pending_queue_redraw();
    }

    pub(super) fn mark_element_only_changed(&mut self, id: ElementId) {
        self.ui_queue_redraw_plan.mark_element(id.as_internal());
        self.ui_queue_redraw_pending = true;
        if self.draining_ui_queue {
            self.request_draw();
        } else {
            // Non-queued mutations should flush immediately on the current frame.
            self.redraw_debounce_until = None;
        }
    }

    pub(super) fn mark_element_and_below_changed(&mut self, id: ElementId) {
        let Some(location) = self.element_location(id) else {
            return;
        };
        self.mark_from_y_changed(location.y);
    }

    pub(super) fn mark_from_y_changed(&mut self, y: u16) {
        self.ui_queue_redraw_plan.mark_from_y(y);
        self.ui_queue_redraw_pending = true;
        if self.draining_ui_queue {
            self.request_draw();
        } else {
            // Non-queued mutations should flush immediately on the current frame.
            self.redraw_debounce_until = None;
        }
    }

    pub(super) fn finish_non_keyboard_redraw(&mut self) {
        if self.sync_layout_redraw_pending {
            self.draw();
            self.sync_layout_redraw_pending = false;
            self.clear_pending_queue_redraw();
            return;
        }
        let _ = self.flush_pending_redraw();
    }

    pub(super) fn finish_terminal_input_redraw(&mut self, full_immediate: bool) {
        if full_immediate || self.sync_layout_redraw_pending {
            self.draw();
            self.sync_layout_redraw_pending = false;
            self.clear_pending_queue_redraw();
            return;
        }
        let _ = self.flush_pending_redraw();
    }

    pub(super) fn flush_pending_queue_redraw_for_keyboard(&mut self) {
        self.redraw_debounce_until = None;
        let _ = self.flush_pending_queue_redraw(true);
    }

    pub(super) fn redraw_keyboard_focused_elements(
        &mut self,
        previous: Option<ElementId>,
        current: Option<ElementId>,
    ) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();

        let ids = focus_order::keyboard_redraw_element_ids(
            previous.map(ElementId::as_internal),
            current.map(ElementId::as_internal),
        );
        for id in ids {
            self.draw_existing_element(id);
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(super) fn redraw_keyboard_current_element(&mut self, current: Option<ElementId>) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();

        if let Some(id) = current.map(ElementId::as_internal) {
            self.draw_existing_element(id);
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(super) fn commit_text_input_redraw(&mut self, id: ElementId, before_text: &str) {
        self.text_input_redraw_committed = true;
        let screen_scroll_before = self.screen_scroll;
        let is_fit_height = self.sync_text_input_viewport_after_edit(id);
        let screen_scroll_changed = is_fit_height && self.screen_scroll != screen_scroll_before;
        let height_changed = self.text_input_height_changed(id, before_text);
        if screen_scroll_changed {
            self.draw();
        } else if height_changed {
            if let Some(anchor_y) = self.element_location(id).map(|location| location.y) {
                self.clear_rows_from(anchor_y);
            }
            self.redraw_text_input_and_below(id);
        } else {
            self.redraw_keyboard_current_element(Some(id));
        }
        self.clear_pending_queue_redraw();
    }

    pub(super) fn redraw_text_input_and_below(&mut self, id: ElementId) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();

        let Some(anchor) = self.element_location(id).map(|loc| loc.y) else {
            return;
        };
        let draw_order = self.elements.focus_order_ids();
        for element_id in draw_order {
            let Some(location) = self.element_location(ElementId::from_internal(element_id)) else {
                continue;
            };
            if location.y >= anchor {
                self.draw_existing_element(element_id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(in crate::runtime) fn draw_existing_element(&mut self, id: usize) {
        #[cfg(debug_draw_do_delay)]
        self.debug_before_draw_existing_element(id);
        self.draw_element(id);
    }

    fn draw_element(&mut self, id: usize) {
        let Some(element) = self.elements.get(id) else {
            return;
        };
        if element.text_input.is_some() {
            self.draw_text_input(id);
        } else if element.is_button() {
            self.draw_button(id);
        } else if matches!(element.height_mode, ElementHeightMode::Fixed(_)) {
            self.draw_text_display(id);
        } else if element.is_fit_static_display() {
            self.draw_wrapped_static_text(id);
        } else {
            self.draw_button(id);
        }
    }

    fn draw_button(&mut self, id: usize) {
        let (location, text, width, style) = match self.elements.get(id) {
            Some(element) => {
                let style = if element.focused {
                    element.style.focused
                } else {
                    element.style.unfocused
                };
                (element.location, element.text.clone(), element.width, style)
            }
            _ => return,
        };

        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        if !terminal_bounds::row_is_visible(y, max_y) || self.is_message_gutter_screen_row(y) {
            return;
        }
        let (_, draw_h) = terminal_bounds::clip_rect(x, y, width.max(1) as i32, 1, max_x, max_y);
        if draw_h <= 0 {
            return;
        }
        let row_cols =
            self.cols_for_printing_respecting_message_gutter(x, max_x, y, max_y) as usize;
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

    fn draw_text_input(&mut self, id: usize) {
        let (
            location,
            width,
            text,
            scroll,
            fixed_viewport,
            cursor,
            selection_anchor,
            style,
        ) = match self.elements.get(id) {
            Some(element) => {
                let Some(input) = element.text_input.as_ref() else {
                    return;
                };
                let style = if input.locked {
                    if element.focused {
                        input.style.focused_locked
                    } else {
                        input.style.unfocused_locked
                    }
                } else if element.focused {
                    input.style.focused_unlocked
                } else {
                    input.style.unfocused_unlocked
                };
                (
                    element.location,
                    element.width.max(1),
                    element.text.clone(),
                    element.scroll,
                    element.fixed_viewport_height(),
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
        let lines = text_wrap::wrapped_lines_for_display(&text, width);
        let total_lines = lines.len();
        let logical_rows = fixed_viewport
            .map(|h| h as i32)
            .unwrap_or_else(|| total_lines as i32);
        let (draw_w, draw_h) =
            terminal_bounds::clip_rect(x, y, width as i32, logical_rows, max_x, max_y);
        if draw_w <= 0 || draw_h <= 0 {
            return;
        }
        let terminal_visible_rows = draw_h as usize;

        let state = TextInputState {
            text: text.clone(),
            cursor,
            selection_anchor,
        };
        let selection = text_input::selection_range(&state);
        let highlight_cells = text_wrap::selection_highlight_cells(&text, selection, width);

        let scroll_viewport = fixed_viewport.unwrap_or_else(|| terminal_visible_rows.max(1));
        let offset = scroll_view::clamp_scroll_offset(scroll, total_lines, scroll_viewport);
        let range = scroll_view::visible_line_range(offset, scroll_viewport, total_lines);
        let content_rows = range.len() as i32;
        if content_rows <= 0 {
            return;
        }
        // Fill only the scrolled viewport rows at y+0.., matching the text draw loop.
        self.fill_solid_viewport_rows(y, x, draw_w, content_rows, base_pair, true);
        for (row, line_idx) in range.enumerate() {
            let line_idx = line_idx;
            let row_y = y + row as i32;
            if self.is_message_gutter_screen_row(row_y) {
                continue;
            }
            let max_cols = self.max_element_row_cols_respecting_message_gutter(
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
            if line_idx < offset || line_idx >= offset + scroll_viewport {
                continue;
            }
            let row_y = y + (line_idx - offset) as i32;
            if self.is_message_gutter_screen_row(row_y) {
                continue;
            }
            let max_cols = self.max_element_row_cols_respecting_message_gutter(
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

    fn draw_wrapped_static_text(&mut self, id: usize) {
        let (location, width, text, style) = match self.elements.get(id) {
            Some(element) => {
                let style = if element.focused {
                    element.style.focused
                } else {
                    element.style.unfocused
                };
                (
                    element.location,
                    element.width.max(1),
                    element.text.clone(),
                    style,
                )
            }
            _ => return,
        };

        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        let lines = text_wrap::wrapped_lines(&text, width);
        let logical_rows = lines.len().max(1) as i32;
        let draw_w = terminal_bounds::clip_rect(x, y, width as i32, logical_rows, max_x, max_y).0;
        if draw_w <= 0 {
            return;
        }

        let pair = self.color_pair(style.fg, style.bg);
        self.fill_solid_rows(y, x, draw_w, logical_rows, pair, true);

        self.win.attron(COLOR_PAIR(pair as u64));
        for (line_idx, line) in lines.iter().enumerate() {
            let row_y = y + line_idx as i32;
            if !terminal_bounds::row_is_visible(row_y, max_y)
                || self.is_message_gutter_screen_row(row_y)
            {
                continue;
            }
            let row_cols = self.max_element_row_cols_respecting_message_gutter(
                x,
                max_x,
                row_y,
                max_y,
                width as i32,
            ) as usize;
            if row_cols == 0 {
                continue;
            }
            self.win.mv(row_y, x);
            let clipped = terminal_bounds::clip_str_to_cols(line, row_cols);
            self.win.addstr(&clipped);
        }
        self.win.attroff(COLOR_PAIR(pair as u64));
    }

    fn draw_text_display(&mut self, id: usize) {
        let (location, width, height, text, scroll, style) = match self.elements.get(id) {
            Some(element) => {
                let ElementHeightMode::Fixed(height) = element.height_mode else {
                    return;
                };
                let style = if element.focused {
                    element.style.focused
                } else {
                    element.style.unfocused
                };
                (
                    element.location,
                    element.width.max(1),
                    height.max(1),
                    element.text.clone(),
                    element.scroll,
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
            if !terminal_bounds::row_is_visible(row_y, max_y)
                || self.is_message_gutter_screen_row(row_y)
            {
                continue;
            }
            let row_cols = self.max_element_row_cols_respecting_message_gutter(
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

    pub(in crate::runtime) fn fill_solid(&self, y: i32, x: i32, w: i32, h: i32, pair: i16) {
        let (max_y, max_x) = self.win.get_max_yx();
        let (w, h) = terminal_bounds::clip_rect(x, y, w, h, max_x, max_y);
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_solid_rows(y, x, w, h, pair, true);
    }

    pub(in crate::runtime) fn fill_solid_overlay(&self, y: i32, x: i32, w: i32, h: i32, pair: i16) {
        let (max_y, max_x) = self.win.get_max_yx();
        let (w, h) = terminal_bounds::clip_rect(x, y, w, h, max_x, max_y);
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_solid_rows(y, x, w, h, pair, false);
    }

    /// Fills `rows` consecutive screen rows starting at anchor `y` (row index 0 → `y`, 1 → `y+1`, …).
    fn fill_solid_viewport_rows(
        &self,
        y: i32,
        x: i32,
        w: i32,
        rows: i32,
        pair: i16,
        skip_message_gutter: bool,
    ) {
        let (max_y, max_x) = self.win.get_max_yx();
        let w = w.min(terminal_bounds::cols_visible_from(x, max_x)).max(0);
        if w <= 0 || rows <= 0 {
            return;
        }
        self.win.attron(COLOR_PAIR(pair as u64));
        for row in 0..rows {
            let row_y = y + row;
            if !terminal_bounds::row_is_visible(row_y, max_y) {
                continue;
            }
            if skip_message_gutter && self.is_message_gutter_screen_row(row_y) {
                continue;
            }
            let row_w = if skip_message_gutter {
                self.cols_for_printing_respecting_message_gutter(x, max_x, row_y, max_y)
                    .min(w)
            } else {
                terminal_bounds::cols_for_printing(x, max_x, row_y, max_y).min(w)
            };
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

    /// Fills a solid background across `logical_rows` rows, clipping width to the terminal
    /// and skipping rows that are off-screen (including above the viewport when scrolled).
    fn fill_solid_rows(
        &self,
        y: i32,
        x: i32,
        w: i32,
        logical_rows: i32,
        pair: i16,
        skip_message_gutter: bool,
    ) {
        let (max_y, max_x) = self.win.get_max_yx();
        let w = w.min(terminal_bounds::cols_visible_from(x, max_x)).max(0);
        if w <= 0 || logical_rows <= 0 {
            return;
        }
        let visible_rows = terminal_bounds::visible_element_line_range(y, logical_rows, max_y);
        self.win.attron(COLOR_PAIR(pair as u64));
        for row in visible_rows {
            let row_y = y + row;
            if skip_message_gutter && self.is_message_gutter_screen_row(row_y) {
                continue;
            }
            let row_w = if skip_message_gutter {
                self.cols_for_printing_respecting_message_gutter(x, max_x, row_y, max_y)
                    .min(w)
            } else {
                terminal_bounds::cols_for_printing(x, max_x, row_y, max_y).min(w)
            };
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

        let Some(input) = self.element_by_id(focused_id) else {
            let _ = curs_set(0);
            return;
        };

        let Some(text_input) = input.text_input.as_ref() else {
            let _ = curs_set(0);
            return;
        };

        if text_input.locked {
            let _ = curs_set(0);
            return;
        }

        let width = input.width.max(1);
        let (line, col) = text_wrap::cursor_display_position(&input.text, text_input.cursor, width);
        let scroll = input.scroll;
        let fixed_viewport = input.fixed_viewport_height();
        let total_lines = text_wrap::wrapped_lines_for_display(&input.text, width).len();
        let (max_y, max_x) = self.win.get_max_yx();
        let x = input.location.x as i32;
        let y = self.scrolled_y(input.location.y as i32);
        let (_, draw_h) =
            terminal_bounds::clip_rect(x, y, width as i32, total_lines as i32, max_x, max_y);
        let terminal_visible_rows = draw_h.max(0) as usize;
        let scroll_viewport = fixed_viewport.unwrap_or_else(|| terminal_visible_rows.max(1));
        let effective_scroll =
            scroll_view::clamp_scroll_offset(scroll, total_lines, scroll_viewport);
        if line < effective_scroll || line >= effective_scroll + scroll_viewport {
            let _ = curs_set(0);
            return;
        }
        let row_y = y + (line - effective_scroll) as i32;
        if !terminal_bounds::row_is_visible(row_y, max_y)
            || self.is_message_gutter_screen_row(row_y)
        {
            let _ = curs_set(0);
            return;
        }
        let max_cols = self.max_element_row_cols_respecting_message_gutter(
            x,
            max_x,
            row_y,
            max_y,
            width as i32,
        );
        if col as i32 >= max_cols {
            let _ = curs_set(0);
            return;
        }
        let _ = curs_set(1);
        self.win.mv(row_y, x + col as i32);
    }

    fn clear_pending_queue_redraw(&mut self) {
        self.ui_queue_redraw_pending = false;
        self.ui_queue_redraw_plan.clear();
        self.redraw_debounce_until = None;
    }

    fn flush_pending_queue_redraw(&mut self, keyboard_immediate: bool) -> bool {
        let now = Instant::now();
        if !ui_redraw::should_flush_debounced_queue_redraw(
            self.ui_queue_redraw_pending,
            self.ui_queue_has_pending(),
            self.redraw_debounce_until,
            now,
        ) && !keyboard_immediate
        {
            return false;
        }
        if !self.ui_queue_redraw_pending {
            return false;
        }
        self.draw_pending_ui_queue_plan();
        self.clear_pending_queue_redraw();
        true
    }

    fn draw_pending_ui_queue_plan(&mut self) {
        let plan = std::mem::take(&mut self.ui_queue_redraw_plan);
        if plan.is_empty() {
            return;
        }

        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();

        if let Some(anchor_y) = plan.redraw_from_y() {
            self.clear_rows_from(anchor_y);
        }

        let draw_order: Vec<usize> = self.elements.focus_order_ids();
        for id in draw_order {
            let Some(location) = self.element_location(ElementId::from_internal(id)) else {
                continue;
            };
            if plan.should_draw(id, location.y) {
                self.draw_existing_element(id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    fn clear_rows_from(&self, anchor_y: u16) {
        let (max_y, max_x) = self.win.get_max_yx();
        let row_y = self.scrolled_y(anchor_y as i32);
        if row_y >= max_y {
            return;
        }
        for y in row_y.max(0)..max_y {
            if self.is_message_gutter_screen_row(y) {
                continue;
            }
            self.win.mv(y, 0);
            let cols = self.cols_for_printing_respecting_message_gutter(0, max_x, y, max_y);
            for _ in 0..cols {
                self.win.addch(' ');
            }
        }
    }

    fn clear_screen_for_draw(&mut self) {
        let (max_y, max_x) = self.win.get_max_yx();
        let max_row = terminal_bounds::content_max_y(max_y);
        for y in 0..=max_row {
            if self.is_message_gutter_screen_row(y) {
                continue;
            }
            self.win.mv(y, 0);
            let cols = self.cols_for_printing_respecting_message_gutter(0, max_x, y, max_y);
            for _ in 0..cols {
                self.win.addch(' ');
            }
        }
    }
}
