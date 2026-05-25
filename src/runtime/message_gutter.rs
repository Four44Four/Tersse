use std::ops::Range;
use std::time::{Duration, Instant};

use pancurses::COLOR_PAIR;

use crate::constants::{
    MSG_GUTTER_BG_COLOR, MSG_GUTTER_DURA_MS, MSG_GUTTER_MAX_HEIGHT, MSG_GUTTER_MULTI_MSG_COLOR,
    MSG_GUTTER_MULTI_MSG_STR, MSG_GUTTER_SIDE,
};
use crate::pure::message_gutter::{
    self, gutter_rows_to_restore, gutter_screen_rows, MessageGutterLine,
};
use crate::pure::terminal_bounds;
use crate::Color;
use crate::ElementId;

use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn message_gutter_expiry_deadline(&self) -> Option<Instant> {
        self.message_gutter_expires_at
    }

    pub(super) fn message_gutter_screen_row_range(&self) -> Range<i32> {
        if !self.message_gutter.visible {
            return 0..0;
        }
        let height = self.message_gutter_layout_height();
        if height == 0 {
            return 0..0;
        }
        let (max_y, _) = self.win.get_max_yx();
        gutter_screen_rows(MSG_GUTTER_SIDE, height, max_y)
    }

    pub(super) fn is_message_gutter_screen_row(&self, screen_y: i32) -> bool {
        self.message_gutter_screen_row_range().contains(&screen_y)
    }

    pub(super) fn cols_for_printing_respecting_message_gutter(
        &self,
        x: i32,
        max_x: i32,
        screen_y: i32,
        terminal_max_y: i32,
    ) -> i32 {
        let cols = terminal_bounds::cols_for_printing(x, max_x, screen_y, terminal_max_y);
        let gutter = self.message_gutter_screen_row_range();
        message_gutter::clip_cols_to_avoid_wrapping_into_row(
            cols,
            x,
            max_x,
            message_gutter::row_printing_wraps_into_gutter_block(gutter, screen_y),
        )
    }

    pub(super) fn max_element_row_cols_respecting_message_gutter(
        &self,
        x: i32,
        max_x: i32,
        row_y: i32,
        terminal_max_y: i32,
        element_width: i32,
    ) -> i32 {
        let cols =
            terminal_bounds::max_element_row_cols(x, max_x, row_y, terminal_max_y, element_width);
        let gutter = self.message_gutter_screen_row_range();
        message_gutter::clip_cols_to_avoid_wrapping_into_row(
            cols,
            x,
            max_x,
            message_gutter::row_printing_wraps_into_gutter_block(gutter, row_y),
        )
    }

    pub(super) fn apply_gutter_message(&mut self, message: String) {
        let now = Instant::now();
        let already_visible = self
            .message_gutter_expires_at
            .is_some_and(|until| now < until && self.message_gutter.visible);
        let previous_height = self.message_gutter.rendered_height;
        // New gutter messages start from the current content viewport position.
        // Any prior top-reveal offset belongs to a previously hidden gutter cycle.
        self.screen_scroll_up_reveal = 0;
        self.message_gutter_reveal_scroll_cap = None;
        self.message_gutter =
            message_gutter::apply_message(&self.message_gutter, message, already_visible);
        self.message_gutter_expires_at = Some(now + Duration::from_millis(MSG_GUTTER_DURA_MS));
        self.refresh_message_gutter_after_change(previous_height);
        self.clamp_screen_scroll_offset();
    }

    pub(super) fn tick_message_gutter_expiry(&mut self) -> bool {
        let Some(until) = self.message_gutter_expires_at else {
            return false;
        };
        if Instant::now() < until {
            return false;
        }
        if !matches!(MSG_GUTTER_SIDE, crate::constants::MsgGutterSide::Top) {
            self.screen_scroll_up_reveal = 0;
        }
        self.hide_message_gutter();
        true
    }

    /// Draws the gutter overlay. Call only when showing, updating, or resizing the gutter message
    /// (`refresh_message_gutter_after_change`), not on screen scroll or full element redraws.
    pub(super) fn draw_message_gutter_overlay(&mut self) {
        if !self.message_gutter.visible {
            return;
        }

        let (max_y, max_x) = self.win.get_max_yx();
        let terminal_width = (max_x + 1).max(1) as usize;
        let lines = message_gutter::layout_message_gutter_lines(
            &self.message_gutter.message,
            self.message_gutter.show_multi_indicator,
            MSG_GUTTER_MULTI_MSG_STR,
            terminal_width,
        );
        let height = lines.len().min(MSG_GUTTER_MAX_HEIGHT.max(1));
        self.message_gutter.rendered_height = height;

        let bg_pair = self.color_pair(Color::White, MSG_GUTTER_BG_COLOR);
        let indicator_pair = self.color_pair(MSG_GUTTER_MULTI_MSG_COLOR, MSG_GUTTER_BG_COLOR);
        let row_range = gutter_screen_rows(MSG_GUTTER_SIDE, height, max_y);

        #[cfg(debug_draw_do_delay)]
        self.debug_before_draw_message_gutter(row_range.start, height as i32, max_x, max_y);

        for (idx, line) in lines.iter().take(height).enumerate() {
            let screen_y = row_range.start + idx as i32;
            if !terminal_bounds::row_is_visible(screen_y, max_y) {
                continue;
            }
            self.draw_message_gutter_row(screen_y, max_x, max_y, line, bg_pair, indicator_pair);
        }
    }

    pub(super) fn hide_message_gutter(&mut self) {
        if !self.message_gutter.visible && self.message_gutter_expires_at.is_none() {
            return;
        }
        let (max_y, _) = self.win.get_max_yx();
        let gutter_height = self.message_gutter.rendered_height;
        let rows_to_restore = if gutter_height > 0 {
            gutter_screen_rows(MSG_GUTTER_SIDE, gutter_height, max_y)
        } else {
            0..0
        };

        self.message_gutter = message_gutter::hide_message(&self.message_gutter);
        self.message_gutter_expires_at = None;
        self.message_gutter_reveal_scroll_cap = None;
        let (base_content_height, full_viewport) = self.full_screen_scroll_bounds();
        let base_max =
            crate::pure::scroll_view::max_scroll_offset(base_content_height, full_viewport);
        match MSG_GUTTER_SIDE {
            crate::constants::MsgGutterSide::Bottom => {
                self.message_gutter.rendered_height = 0;
                if self.screen_scroll > base_max {
                    // Preserve scroll position after hiding so post-hide padding remains visible.
                    // Future scroll-up ratchets this cap down.
                    self.message_gutter_reveal_scroll_cap = Some(self.screen_scroll);
                }
            }
            crate::constants::MsgGutterSide::Top => {
                self.message_gutter.rendered_height = 0;
                if self.screen_scroll > base_max {
                    self.message_gutter_reveal_scroll_cap = Some(self.screen_scroll);
                } else if self.screen_scroll_up_reveal > 0 {
                    self.message_gutter_reveal_scroll_cap = Some(self.screen_scroll_up_reveal);
                }
            }
        }

        if !rows_to_restore.is_empty() {
            self.restore_screen_rows(rows_to_restore);
        }
        self.fill_scroll_padding_rows();
        self.win.refresh();
    }

    pub(super) fn message_gutter_layout_height(&self) -> usize {
        if !self.message_gutter.visible {
            return 0;
        }
        let (_, max_x) = self.win.get_max_yx();
        let terminal_width = (max_x + 1).max(1) as usize;
        let layout_height = message_gutter::message_gutter_height(
            &self.message_gutter.message,
            self.message_gutter.show_multi_indicator,
            MSG_GUTTER_MULTI_MSG_STR,
            terminal_width,
            MSG_GUTTER_MAX_HEIGHT,
        );
        layout_height.max(self.message_gutter.rendered_height)
    }

    pub(super) fn fill_scroll_padding_rows(&mut self) {
        let (base_content_height, full_viewport) = self.full_screen_scroll_bounds();
        let (max_y, max_x) = self.win.get_max_yx();
        let pair = self.color_pair(Color::Default, Color::Default);

        if matches!(MSG_GUTTER_SIDE, crate::constants::MsgGutterSide::Top)
            && message_gutter::screen_scroll_shows_top_padding(
                self.screen_scroll,
                self.screen_scroll_up_reveal,
                self.message_gutter.visible,
            )
        {
            let rows = message_gutter::top_padding_screen_rows(
                self.screen_scroll,
                self.screen_scroll_up_reveal,
            );
            for screen_y in rows {
                self.fill_solid_overlay(screen_y, 0, max_x + 1, 1, pair);
            }
        }

        if !message_gutter::screen_scroll_shows_padding(
            self.screen_scroll,
            base_content_height,
            full_viewport,
            self.message_gutter.visible,
        ) {
            return;
        }
        let rows = message_gutter::padding_screen_rows(
            self.screen_scroll,
            base_content_height,
            max_y,
        );
        for screen_y in rows {
            if self.is_message_gutter_screen_row(screen_y) {
                continue;
            }
            self.fill_solid_overlay(screen_y, 0, max_x + 1, 1, pair);
        }
    }

    fn refresh_message_gutter_after_change(&mut self, previous_height: usize) {
        let (max_y, max_x) = self.win.get_max_yx();
        let terminal_width = (max_x + 1).max(1) as usize;
        let new_height = message_gutter::message_gutter_height(
            &self.message_gutter.message,
            self.message_gutter.show_multi_indicator,
            MSG_GUTTER_MULTI_MSG_STR,
            terminal_width,
            MSG_GUTTER_MAX_HEIGHT,
        );
        let restore = gutter_rows_to_restore(MSG_GUTTER_SIDE, previous_height, new_height, max_y);
        if !restore.is_empty() {
            self.restore_screen_rows(restore);
        }
        self.draw_message_gutter_overlay();
        self.win.refresh();
    }

    fn draw_message_gutter_row(
        &mut self,
        screen_y: i32,
        max_x: i32,
        max_y: i32,
        line: &MessageGutterLine,
        message_pair: i16,
        indicator_pair: i16,
    ) {
        let row_cols =
            self.cols_for_printing_respecting_message_gutter(0, max_x, screen_y, max_y) as usize;
        if row_cols == 0 {
            return;
        }

        self.fill_solid_overlay(screen_y, 0, row_cols as i32, 1, message_pair);

        let message_cols = line.message_text.chars().count();
        let indicator_cols = line
            .indicator_text
            .as_deref()
            .map(str::chars)
            .map(|chars| chars.count())
            .unwrap_or(0);
        let draw_cols = message_cols + indicator_cols;
        if draw_cols == 0 {
            return;
        }

        self.win.mv(screen_y, 0);
        if !line.message_text.is_empty() {
            self.win.attron(COLOR_PAIR(message_pair as pancurses::chtype));
            let clipped = terminal_bounds::clip_str_to_cols(&line.message_text, row_cols);
            self.win.addstr(&clipped);
            self.win.attroff(COLOR_PAIR(message_pair as pancurses::chtype));
        }

        if let Some(indicator) = &line.indicator_text {
            let indicator_x = message_cols as i32;
            if indicator_x < row_cols as i32 {
                self.win.attron(COLOR_PAIR(indicator_pair as pancurses::chtype));
                self.win.mv(screen_y, indicator_x);
                let remaining = row_cols.saturating_sub(message_cols);
                let clipped = terminal_bounds::clip_str_to_cols(indicator, remaining);
                self.win.addstr(&clipped);
                self.win.attroff(COLOR_PAIR(indicator_pair as pancurses::chtype));
            }
        }
    }

    fn restore_screen_rows(&mut self, screen_rows: Range<i32>) {
        if screen_rows.is_empty() {
            return;
        }

        let (max_y, max_x) = self.win.get_max_yx();
        for screen_y in screen_rows.clone() {
            if !terminal_bounds::row_is_visible(screen_y, max_y) {
                continue;
            }
            self.win.mv(screen_y, 0);
            let cols = self.cols_for_printing_respecting_message_gutter(0, max_x, screen_y, max_y);
            for _ in 0..cols {
                self.win.addch(' ');
            }
        }

        let draw_order: Vec<usize> = self.elements.focus_order_ids();
        for id in draw_order {
            let element_id = ElementId::from_internal(id);
            let Some(location) = self.element_location(element_id) else {
                continue;
            };
            let height = self.cached_heights.get(&id).copied().unwrap_or(1);
            if message_gutter::element_row_intersects_gutter_screen_rows(
                location.y,
                height,
                self.screen_scroll,
                self.screen_scroll_up_reveal,
                screen_rows.clone(),
            ) {
                self.draw_existing_element(id);
            }
        }
    }
}
