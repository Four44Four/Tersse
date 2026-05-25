use crate::constants::MSG_GUTTER_SIDE;
use crate::pure::message_gutter;
use crate::pure::screen_scroll;
use crate::terminal_input::TerminalKey;

use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn handle_screen_scroll(&mut self, key: TerminalKey) -> bool {
        let (base_content_height, viewport_height) = self.screen_scroll_bounds();
        let reveal_cap = self.message_gutter_reveal_scroll_cap;
        match key {
            TerminalKey::ShiftUp => {
                let previous = self.screen_scroll;
                self.screen_scroll = screen_scroll::scroll_screen_up(self.screen_scroll);
                if self.screen_scroll < previous
                    && !self.message_gutter.visible
                    && self.message_gutter_reveal_scroll_cap.is_some()
                {
                    let (_, full_viewport) = self.full_screen_scroll_bounds();
                    let base_max = crate::pure::scroll_view::max_scroll_offset(
                        base_content_height,
                        full_viewport,
                    );
                    self.message_gutter_reveal_scroll_cap =
                        message_gutter::ratchet_gutter_scroll_cap_on_up(
                            self.message_gutter_reveal_scroll_cap,
                            self.screen_scroll,
                            base_max,
                        );
                }
                true
            }
            TerminalKey::ShiftDown => {
                self.screen_scroll = message_gutter::scroll_screen_down_with_gutter(
                    self.screen_scroll,
                    base_content_height,
                    viewport_height,
                    reveal_cap,
                );
                true
            }
            _ => false,
        }
    }

    pub(super) fn clamp_screen_scroll_offset(&mut self) {
        let (base_content_height, viewport_height) = self.screen_scroll_bounds();
        self.screen_scroll = message_gutter::clamp_screen_scroll_with_gutter(
            self.screen_scroll,
            base_content_height,
            viewport_height,
            self.message_gutter_reveal_scroll_cap,
        );
    }

    /// Content height and viewport used for scroll clamping (shorter when gutter is visible).
    pub(super) fn screen_scroll_bounds(&self) -> (usize, usize) {
        let (content_height, full_viewport) = self.full_screen_scroll_bounds();
        let viewport = message_gutter::viewport_height_for_screen_scroll(
            full_viewport,
            self.message_gutter.visible,
            self.message_gutter_layout_height(),
            MSG_GUTTER_SIDE,
        );
        (content_height, viewport)
    }

    /// Content height and the full terminal viewport (gutter does not reduce this).
    pub(super) fn full_screen_scroll_bounds(&self) -> (usize, usize) {
        let (max_y, _) = self.win.get_max_yx();
        let viewport_height = screen_scroll::screen_viewport_height(max_y);
        let spans = self
            .elements
            .iter()
            .map(|element| {
                let y = element.location.y;
                let height = self.cached_heights.get(&element.id()).copied().unwrap_or(1);
                (y, height)
            })
            .collect::<Vec<_>>();
        let content_height = screen_scroll::screen_content_height(&spans);
        (content_height, viewport_height)
    }

    pub(super) fn scrolled_y(&self, logical_y: i32) -> i32 {
        screen_scroll::apply_scroll_to_y(logical_y, self.screen_scroll)
    }
}
