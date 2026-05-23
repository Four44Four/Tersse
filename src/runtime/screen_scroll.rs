use crate::pure::screen_scroll;
use crate::terminal_input::TerminalKey;

use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn handle_screen_scroll(&mut self, key: TerminalKey) -> bool {
        let (content_height, viewport_height) = self.screen_scroll_bounds();
        match key {
            TerminalKey::ShiftUp => {
                self.screen_scroll = screen_scroll::scroll_screen_up(self.screen_scroll);
                true
            }
            TerminalKey::ShiftDown => {
                self.screen_scroll = screen_scroll::scroll_screen_down(
                    self.screen_scroll,
                    content_height,
                    viewport_height,
                );
                true
            }
            _ => false,
        }
    }

    pub(super) fn clamp_screen_scroll_offset(&mut self) {
        let (content_height, viewport_height) = self.screen_scroll_bounds();
        self.screen_scroll =
            screen_scroll::clamp_screen_scroll(self.screen_scroll, content_height, viewport_height);
    }

    pub(super) fn screen_scroll_bounds(&self) -> (usize, usize) {
        let (max_y, _) = self.win.get_max_yx();
        let viewport_height = screen_scroll::screen_viewport_height(max_y);
        let spans = self.elements.iter().map(|element| {
                let y = match element {
                    super::types::RuntimeElement::Button(button) => button.button.location.y,
                    super::types::RuntimeElement::TextInput(input) => input.location.y,
                    super::types::RuntimeElement::TextDisplay(display) => display.location.y,
                };
                let height = self
                    .cached_heights
                    .get(element.id())
                    .copied()
                    .unwrap_or(1);
                (y, height)
            })
            .collect::<Vec<_>>();
        let content_height =
            screen_scroll::screen_content_height(self.title.is_some(), &spans);
        (content_height, viewport_height)
    }

    pub(super) fn scrolled_y(&self, logical_y: i32) -> i32 {
        screen_scroll::apply_scroll_to_y(logical_y, self.screen_scroll)
    }
}
