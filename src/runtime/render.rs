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
        self.erase_content_for_draw();
        self.paint_visible_elements();
        // A full-screen draw supersedes any incremental queue redraw plan (e.g. marks
        // accumulated while creating elements before the first frame).
        self.clear_pending_queue_redraw();
    }

    /// Repaints on-screen elements after screen scroll without redrawing off-screen elements.
    pub(super) fn redraw_after_screen_scroll(&mut self) {
        self.erase_content_for_draw();
        self.paint_visible_elements();
        self.clear_pending_queue_redraw();
    }

    fn erase_content_for_draw(&mut self) {
        if self.message_gutter.visible {
            self.clear_screen_for_draw();
        } else {
            self.win.erase();
        }
    }

    fn paint_visible_elements(&mut self) {
        self.auto_reflow_for_dynamic_heights();
        self.clamp_screen_scroll_offset();
        self.sync_focus_flags();

        let draw_order: Vec<usize> = self.elements.focus_order_ids();
        for id in draw_order {
            if self.element_intersects_terminal_viewport(id) {
                self.draw_existing_element(id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        // Message gutter overlay is drawn only on apply/update (refresh) or hide; see
        // specifications/message_gutter.txt. clear_screen_for_draw already skips gutter rows.
        self.win.refresh();
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
            if self.element_intersects_terminal_viewport(id) {
                self.draw_existing_element(id);
            }
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
            if self.element_intersects_terminal_viewport(id) {
                self.draw_existing_element(id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(super) fn commit_text_input_redraw(&mut self, id: ElementId, before_text: &str) {
        self.text_input_redraw_committed = true;
        let screen_scroll_before = self.screen_scroll;
        let (is_fit_height, layout_height_changed) = self.sync_text_input_viewport_after_edit(id);
        let screen_scroll_changed = is_fit_height && self.screen_scroll != screen_scroll_before;
        let height_changed =
            layout_height_changed || self.text_input_height_changed(id, before_text);
        if screen_scroll_changed || height_changed {
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
            if location.y >= anchor && self.element_intersects_terminal_viewport(element_id) {
                self.draw_existing_element(element_id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(in crate::runtime) fn element_intersects_terminal_viewport(&self, id: usize) -> bool {
        let Some(element) = self.elements.get(id) else {
            return false;
        };
        let logical_rows = self
            .cached_heights
            .get(&id)
            .copied()
            .unwrap_or_else(|| self.element_render_height(element)) as i32;
        let (_, max_y) = self.win.get_max_yx();
        let anchor_y = self.scrolled_y(element.location.y as i32);
        terminal_bounds::element_intersects_terminal_viewport(anchor_y, logical_rows, max_y)
    }

    pub(in crate::runtime) fn draw_existing_element(&mut self, id: usize) {
        #[cfg(debug_draw_do_delay)]
        self.debug_before_draw_existing_element(id);
        self.draw_element(id);
    }

    fn draw_element(&mut self, id: usize) {
        let (
            location,
            width,
            text,
            scroll,
            height_mode,
            focused,
            base_style,
            text_input_state,
            text_input_style,
            selection_style,
        ) = match self.elements.get(id) {
            Some(element) => {
                let (text_input_state, text_input_style, selection_style) =
                    if let Some(input) = element.text_input.as_ref() {
                        let active_style = if input.locked {
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
                            Some((input.cursor, input.selection_anchor)),
                            Some(active_style),
                            Some(input.style.selection),
                        )
                    } else {
                        (None, None, None)
                    };
                (
                    element.location,
                    element.width.max(1),
                    element.text.clone(),
                    element.scroll,
                    element.height_mode,
                    element.focused,
                    element.style,
                    text_input_state,
                    text_input_style,
                    selection_style,
                )
            }
            _ => return,
        };

        let active_style = if let Some(style) = text_input_style {
            style
        } else if focused {
            base_style.focused
        } else {
            base_style.unfocused
        };
        let base_pair = self.color_pair(active_style.fg, active_style.bg);
        let selection_pair =
            selection_style.map(|style| self.color_pair(style.fg, style.bg));

        let mut lines = if text_input_state.is_some() {
            text_wrap::wrapped_lines_for_display(&text, width)
        } else {
            text_wrap::wrapped_lines(&text, width)
        };
        if lines.is_empty() {
            lines.push(String::new());
        }
        let total_lines = lines.len();

        let logical_rows = match height_mode {
            ElementHeightMode::Fixed(height) => height.max(1),
            ElementHeightMode::FitContent => total_lines.max(1),
        };
        let (max_y, max_x) = self.win.get_max_yx();
        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        if !terminal_bounds::element_intersects_terminal_viewport(y, logical_rows as i32, max_y) {
            return;
        }
        let (draw_w, draw_h) =
            terminal_bounds::clip_rect(x, y, width as i32, logical_rows as i32, max_x, max_y);
        if draw_w <= 0 || draw_h <= 0 {
            return;
        }

        match height_mode {
            ElementHeightMode::Fixed(_) => self.fill_solid(y, x, draw_w, draw_h, base_pair),
            ElementHeightMode::FitContent => {
                self.fill_solid_rows(y, x, draw_w, logical_rows as i32, base_pair, true)
            }
        }

        let (visible_lines, line_to_screen_row): (Vec<usize>, Box<dyn Fn(usize) -> i32>) =
            match height_mode {
                ElementHeightMode::Fixed(height) => {
                    let viewport_rows = height.max(1);
                    let offset =
                        scroll_view::clamp_scroll_offset(scroll, total_lines, viewport_rows);
                    (
                        scroll_view::visible_line_range(offset, viewport_rows, total_lines)
                            .collect(),
                        Box::new(move |line_idx| (line_idx.saturating_sub(offset)) as i32),
                    )
                }
                ElementHeightMode::FitContent => (
                    terminal_bounds::visible_element_line_range(y, logical_rows as i32, max_y)
                        .map(|row| row as usize)
                        .collect(),
                    Box::new(move |line_idx| line_idx as i32),
                ),
            };

        let highlight_cells = text_input_state.map(|(cursor, selection_anchor)| {
            let state = TextInputState {
                text: text.clone(),
                cursor,
                selection_anchor,
            };
            let selection = text_input::selection_range(&state);
            text_wrap::selection_highlight_cells(&text, selection, width)
        });

        if highlight_cells.is_none() {
            self.win.attron(COLOR_PAIR(base_pair as u64));
        }
        for line_idx in visible_lines {
            let row_y = y + line_to_screen_row(line_idx);
            if self.is_message_gutter_screen_row(row_y)
                || !terminal_bounds::row_is_visible(row_y, max_y)
            {
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

            if let (Some(cells), Some(selection_pair)) =
                (highlight_cells.as_ref(), selection_pair)
            {
                for (col, ch) in line.chars().enumerate() {
                    if col >= max_cols {
                        break;
                    }
                    let pair = if cells.contains(&(line_idx, col)) {
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
                    if !cells.contains(&(line_idx, col)) {
                        continue;
                    }
                    self.win.attron(COLOR_PAIR(selection_pair as u64));
                    self.win.mv(row_y, x + col as i32);
                    self.win.addch(' ');
                    self.win.attroff(COLOR_PAIR(selection_pair as u64));
                }
            } else {
                self.win.mv(row_y, x);
                let clipped = terminal_bounds::clip_str_to_cols(line, max_cols);
                self.win.addstr(&clipped);
            }
        }
        if highlight_cells.is_none() {
            self.win.attroff(COLOR_PAIR(base_pair as u64));
        }
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
        let scroll_viewport = fixed_viewport.unwrap_or_else(|| {
            self.text_input_terminal_visible_rows(input.location.y, total_lines)
                .max(1)
        });
        let row_y = if fixed_viewport.is_some() {
            let effective_scroll =
                scroll_view::clamp_scroll_offset(scroll, total_lines, scroll_viewport);
            if line < effective_scroll || line >= effective_scroll + scroll_viewport {
                let _ = curs_set(0);
                return;
            }
            y + (line - effective_scroll) as i32
        } else {
            let visible_lines =
                terminal_bounds::visible_element_line_range(y, total_lines as i32, max_y);
            if line < visible_lines.start as usize || line >= visible_lines.end as usize {
                let _ = curs_set(0);
                return;
            }
            y + line as i32
        };
        if self.is_message_gutter_screen_row(row_y)
            || !terminal_bounds::row_is_visible(row_y, max_y)
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
            if plan.should_draw(id, location.y)
                && self.element_intersects_terminal_viewport(id)
            {
                self.draw_existing_element(id);
            }
        }

        self.draw_cursor_for_active_text_input();
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    /// Clears all terminal cells within an element's bounds.
    pub(super) fn clear_element_occupied_space(
        &self,
        bounds: crate::pure::element_placement::ElementBounds,
    ) {
        if bounds.height == 0 {
            return;
        }
        let (max_y, max_x) = self.win.get_max_yx();
        let x = bounds.x as i32;
        let y = self.scrolled_y(bounds.y as i32);
        let row_range =
            terminal_bounds::visible_element_line_range(y, bounds.height as i32, max_y);
        for line_idx in row_range {
            let row_y = y + line_idx;
            if !terminal_bounds::row_is_visible(row_y, max_y)
                || self.is_message_gutter_screen_row(row_y)
            {
                continue;
            }
            let cols = terminal_bounds::max_element_row_cols(
                x,
                max_x,
                row_y,
                max_y,
                bounds.width as i32,
            );
            for col in 0..cols {
                self.win.mv(row_y, x + col);
                self.win.addch(' ');
            }
        }
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
