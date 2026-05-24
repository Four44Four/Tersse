use std::time::Duration;

use crate::constants::{DebugDrawDelayColor, DEBUG_DRAW_DELAY_COLOR, DEBUG_DRAW_DELAY_MS};
use crate::pure::terminal_bounds;
use crate::Color;

use super::layout::{
    render_height_for_button, render_height_for_text_display, render_height_for_text_input_text,
};
use super::types::RuntimeElement;
use super::RuntimeUi;

impl RuntimeUi {
    pub(super) fn debug_before_draw_message_gutter(
        &mut self,
        screen_y: i32,
        height: i32,
        max_x: i32,
        max_y: i32,
    ) {
        let (w, h) = terminal_bounds::clip_rect(0, screen_y, max_x + 1, height, max_x, max_y);
        if w <= 0 || h <= 0 {
            return;
        }

        let color = debug_draw_delay_color(DEBUG_DRAW_DELAY_COLOR);
        let pair = self.color_pair(color, color);
        self.fill_solid_overlay(screen_y, 0, w, h, pair);
        self.win.refresh();
        std::thread::sleep(Duration::from_millis(DEBUG_DRAW_DELAY_MS));
    }

    pub(super) fn debug_before_draw_existing_element(&mut self, id: usize) {
        let Some((x, y, w, h)) = self.existing_element_screen_rect(id) else {
            return;
        };
        if w <= 0 || h <= 0 {
            return;
        }

        let color = debug_draw_delay_color(DEBUG_DRAW_DELAY_COLOR);
        let pair = self.color_pair(color, color);
        self.fill_solid(y, x, w, h, pair);
        self.win.refresh();
        std::thread::sleep(Duration::from_millis(DEBUG_DRAW_DELAY_MS));
    }

    fn existing_element_screen_rect(&self, id: usize) -> Option<(i32, i32, i32, i32)> {
        let element = self.elements.get(id)?;
        let (location, width, height) = match element {
            RuntimeElement::Button(button) => (
                button.button.location,
                button.button.width.max(1),
                render_height_for_button(),
            ),
            RuntimeElement::TextInput(input) => {
                let width = input.field.width.max(1);
                let height = render_height_for_text_input_text(&input.field.text, width);
                (input.location, width, height)
            }
            RuntimeElement::TextDisplay(display) => (
                display.location,
                display.width.max(1),
                render_height_for_text_display(display.height),
            ),
        };

        let x = location.x as i32;
        let y = self.scrolled_y(location.y as i32);
        let w = width.max(1) as i32;
        let h = height.max(1) as i32;
        Some((x, y, w, h))
    }
}

fn debug_draw_delay_color(color: DebugDrawDelayColor) -> Color {
    match color {
        DebugDrawDelayColor::Red => Color::Red,
    }
}
