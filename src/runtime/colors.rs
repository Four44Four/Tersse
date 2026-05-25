use crate::Color;

use super::TersseUi;

impl TersseUi {
    pub(super) fn color_pair(&mut self, fg: Color, bg: Color) -> i16 {
        let fg_code = terminal_color_code(fg);
        let bg_code = terminal_color_code(bg);
        let key = (fg_code, bg_code);
        if let Some(pair) = self.pair_cache.get(&key) {
            return *pair;
        }
        let pair = self.next_pair_id;
        pancurses::init_pair(pair, fg_code, bg_code);
        self.pair_cache.insert(key, pair);
        self.next_pair_id = self.next_pair_id.saturating_add(1);
        pair
    }
}

pub(crate) fn terminal_color_code(color: Color) -> i16 {
    match color {
        Color::Default => -1,
        Color::Black => pancurses::COLOR_BLACK,
        Color::Red => pancurses::COLOR_RED,
        Color::Green => pancurses::COLOR_GREEN,
        Color::Yellow => pancurses::COLOR_YELLOW,
        Color::Blue => pancurses::COLOR_BLUE,
        Color::Magenta => pancurses::COLOR_MAGENTA,
        Color::Cyan => pancurses::COLOR_CYAN,
        Color::White => pancurses::COLOR_WHITE,
    }
}
