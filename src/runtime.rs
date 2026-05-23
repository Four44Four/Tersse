use std::collections::HashMap;
use std::time::Duration;

use pancurses::{curs_set, endwin, initscr, noecho, Window, COLOR_PAIR};

use crate::clipboard;
use crate::pure::scroll_view;
use crate::pure::terminal_bounds;
use crate::pure::text_input::{self, TextInputState};
use crate::pure::text_wrap;
use crate::terminal_input::{self, TerminalKey};
use crate::{create_button, create_text_display_element, create_text_input_field_element, Button, Color, Location, ScreenTitle, TextDisplayElement, TextInputField, TitleAlignment};

#[derive(Clone, Copy)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
}

#[derive(Clone, Copy)]
pub struct FocusStyle {
    pub focused: Style,
    pub unfocused: Style,
}

#[derive(Clone, Copy)]
pub struct TextInputStyle {
    pub focused_unlocked: Style,
    pub unfocused_unlocked: Style,
    pub focused_locked: Style,
    pub unfocused_locked: Style,
    pub selection: Style,
}

pub struct ButtonConfig {
    pub id: String,
    pub label: String,
    pub width: usize,
    pub location: Location,
    pub focus_index: usize,
    pub style: FocusStyle,
    pub on_press: ButtonHandler,
}

pub struct TextInputConfig {
    pub id: String,
    pub width: usize,
    pub location: Location,
    pub focus_index: usize,
    pub style: TextInputStyle,
    pub locked: bool,
    pub initial_text: String,
}

pub struct TextDisplayConfig {
    pub id: String,
    pub location: Location,
    pub width: usize,
    pub height: usize,
    pub focus_index: usize,
    pub style: FocusStyle,
    pub initial_text: String,
}

pub enum ElementConfig {
    Button(ButtonConfig),
    TextInput(TextInputConfig),
    TextDisplay(TextDisplayConfig),
}

pub enum UiEvent {
    None,
    Quit,
}

pub type ButtonHandler = Box<dyn FnMut(&mut RuntimeUi) + 'static>;

struct ButtonElement {
    id: String,
    focus_index: usize,
    button: Button,
    style: FocusStyle,
    on_press: Option<ButtonHandler>,
}

struct TextInputElement {
    id: String,
    focus_index: usize,
    location: Location,
    field: TextInputField,
    cursor: usize,
    selection_anchor: Option<usize>,
    style: TextInputStyle,
}

struct TextDisplayRuntimeElement {
    id: String,
    focus_index: usize,
    location: Location,
    width: usize,
    height: usize,
    scroll: usize,
    display: TextDisplayElement,
    style: FocusStyle,
}

enum RuntimeElement {
    Button(ButtonElement),
    TextInput(TextInputElement),
    TextDisplay(TextDisplayRuntimeElement),
}

pub struct RuntimeUi {
    win: Window,
    title: Option<ScreenTitle>,
    elements: Vec<RuntimeElement>,
    focused_position: usize,
    pair_cache: HashMap<(i16, i16), i16>,
    next_pair_id: i16,
    cached_heights: HashMap<String, usize>,
}

impl RuntimeUi {
    pub fn new() -> Self {
        let _ = terminal_input::enter_raw_mode();
        let win = initscr();
        noecho();
        let _ = curs_set(0);
        pancurses::start_color();
        pancurses::use_default_colors();

        Self {
            win,
            title: None,
            elements: Vec::new(),
            focused_position: 0,
            pair_cache: HashMap::new(),
            next_pair_id: 1,
            cached_heights: HashMap::new(),
        }
    }

    pub fn set_title(&mut self, title: ScreenTitle) {
        self.title = Some(title);
    }

    pub fn clear_title(&mut self) {
        self.title = None;
    }

    pub fn upsert_button(&mut self, config: ButtonConfig) {
        let ButtonConfig {
            id,
            label,
            width,
            location,
            focus_index,
            style,
            on_press,
        } = config;
        if let Some(idx) = self.idx_of(&id) {
            let button = create_button(
                location,
                label,
                width,
                style.unfocused.bg,
                style.unfocused.fg,
                Box::new(|| {}),
            );
            self.elements[idx] = RuntimeElement::Button(ButtonElement {
                id,
                focus_index,
                button,
                style,
                on_press: Some(on_press),
            });
        } else {
            self.elements.push(RuntimeElement::Button(ButtonElement {
                id: id.clone(),
                focus_index,
                button: create_button(
                    location,
                    label,
                    width,
                    style.unfocused.bg,
                    style.unfocused.fg,
                    Box::new(|| {}),
                ),
                style,
                on_press: Some(on_press),
            }));
        }
        self.sync_focus_position();
        self.refresh_height_cache();
    }

    pub fn button_width(&self, id: &str) -> Option<usize> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.width),
            _ => None,
        }
    }

    pub fn upsert_text_input(&mut self, config: TextInputConfig) {
        let mut field = create_text_input_field_element(config.width);
        field.locked = config.locked;
        field.text = config.initial_text;

        if let Some(idx) = self.idx_of(&config.id) {
            self.elements[idx] = RuntimeElement::TextInput(TextInputElement {
                id: config.id,
                focus_index: config.focus_index,
                location: config.location,
                cursor: 0,
                selection_anchor: None,
                field,
                style: config.style,
            });
        } else {
            self.elements.push(RuntimeElement::TextInput(TextInputElement {
                id: config.id,
                focus_index: config.focus_index,
                location: config.location,
                cursor: 0,
                selection_anchor: None,
                field,
                style: config.style,
            }));
        }
        self.sync_focus_position();
        self.refresh_height_cache();
    }

    pub fn upsert_text_display(&mut self, config: TextDisplayConfig) {
        let display = create_text_display_element(config.initial_text);
        if let Some(idx) = self.idx_of(&config.id) {
            self.elements[idx] = RuntimeElement::TextDisplay(TextDisplayRuntimeElement {
                id: config.id,
                focus_index: config.focus_index,
                location: config.location,
                width: config.width.max(1),
                height: config.height.max(1),
                scroll: 0,
                display,
                style: config.style,
            });
        } else {
            self.elements.push(RuntimeElement::TextDisplay(TextDisplayRuntimeElement {
                id: config.id,
                focus_index: config.focus_index,
                location: config.location,
                width: config.width.max(1),
                height: config.height.max(1),
                scroll: 0,
                display,
                style: config.style,
            }));
        }
        self.sync_focus_position();
        self.refresh_height_cache();
    }

    pub fn upsert_and_reflow(&mut self, config: ElementConfig) {
        let id = match &config {
            ElementConfig::Button(cfg) => cfg.id.clone(),
            ElementConfig::TextInput(cfg) => cfg.id.clone(),
            ElementConfig::TextDisplay(cfg) => cfg.id.clone(),
        };
        let old_height = self.element_render_height_by_id(&id).unwrap_or(0);
        let anchor_y = self.element_location(&id).map(|loc| loc.y).or_else(|| match &config {
            ElementConfig::Button(cfg) => Some(cfg.location.y),
            ElementConfig::TextInput(cfg) => Some(cfg.location.y),
            ElementConfig::TextDisplay(cfg) => Some(cfg.location.y),
        });

        match config {
            ElementConfig::Button(cfg) => self.upsert_button(cfg),
            ElementConfig::TextInput(cfg) => self.upsert_text_input(cfg),
            ElementConfig::TextDisplay(cfg) => self.upsert_text_display(cfg),
        }

        let new_height = self.element_render_height_by_id(&id).unwrap_or(0);
        let delta = new_height as i32 - old_height as i32;
        if delta != 0 {
            if let Some(y) = anchor_y {
                let min_y = if old_height == 0 {
                    y
                } else {
                    y.saturating_add(old_height as u16)
                };
                self.shift_elements_from_min_y(&id, min_y, delta);
            }
        }
        self.refresh_height_cache();
    }

    pub fn remove_and_reflow(&mut self, id: &str) -> bool {
        let Some(location) = self.element_location(id) else {
            return false;
        };
        let removed_height = self.element_render_height_by_id(id).unwrap_or(0);
        if !self.remove_element(id) {
            return false;
        }
        if removed_height > 0 {
            let min_y = location.y.saturating_add(removed_height as u16);
            self.shift_elements_from_min_y(id, min_y, -(removed_height as i32));
        }
        self.refresh_height_cache();
        true
    }

    pub fn remove_element(&mut self, id: &str) -> bool {
        if let Some(idx) = self.idx_of(id) {
            self.elements.remove(idx);
            self.sync_focus_position();
            self.cached_heights.remove(id);
            true
        } else {
            false
        }
    }

    pub fn element_location(&self, id: &str) -> Option<Location> {
        match self.element_by_id(id) {
            Some(RuntimeElement::Button(button)) => Some(button.button.location),
            Some(RuntimeElement::TextInput(input)) => Some(input.location),
            Some(RuntimeElement::TextDisplay(display)) => Some(display.location),
            None => None,
        }
    }

    pub fn set_focus_index(&mut self, id: &str, focus_index: usize) -> bool {
        if let Some(element) = self.element_mut_by_id(id) {
            match element {
                RuntimeElement::Button(button) => button.focus_index = focus_index,
                RuntimeElement::TextInput(input) => input.focus_index = focus_index,
                RuntimeElement::TextDisplay(display) => display.focus_index = focus_index,
            }
            self.sync_focus_position();
            true
        } else {
            false
        }
    }

    pub fn set_element_location(&mut self, id: &str, location: Location) -> bool {
        if let Some(element) = self.element_mut_by_id(id) {
            match element {
                RuntimeElement::Button(button) => button.button.location = location,
                RuntimeElement::TextInput(input) => input.location = location,
                RuntimeElement::TextDisplay(display) => display.location = location,
            }
            true
        } else {
            false
        }
    }

    pub fn set_text_display_dimensions(&mut self, id: &str, width: usize, height: usize) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.width = width.max(1);
            display.height = height.max(1);
            true
        } else {
            false
        }
    }

    pub fn set_text_display_text(&mut self, id: &str, text: impl Into<String>) -> bool {
        if let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(id) {
            display.display.text = text.into();
            display.scroll = 0;
            true
        } else {
            false
        }
    }

    pub fn read_text_input(&self, id: &str) -> Option<String> {
        match self.element_by_id(id) {
            Some(RuntimeElement::TextInput(input)) => Some(input.field.text.clone()),
            _ => None,
        }
    }

    pub fn set_text_input_text(&mut self, id: &str, text: impl Into<String>) -> bool {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = text.into();
            input.cursor = input.field.text.chars().count();
            input.selection_anchor = None;
            true
        } else {
            false
        }
    }

    pub fn set_text_input_lock_status(&mut self, id: &str, locked: bool) -> bool {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.locked = locked;
            if locked {
                input.selection_anchor = None;
            }
            true
        } else {
            false
        }
    }

    pub fn draw(&mut self) {
        self.auto_reflow_for_dynamic_heights();
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

    /// Draw one frame and process one input event.
    ///
    /// Returns `false` when the runtime receives a quit key.
    pub fn run_frame(&mut self, timeout: Duration) -> bool {
        self.draw();
        !matches!(self.poll_event(timeout), UiEvent::Quit)
    }

    pub fn poll_event(&mut self, timeout: Duration) -> UiEvent {
        match terminal_input::poll_key(timeout) {
            Ok(Some(key)) => self.handle_key(key),
            Ok(None) => UiEvent::None,
            Err(_) => UiEvent::Quit,
        }
    }

    fn draw_title(&mut self, title: ScreenTitle) {
        let pair = self.color_pair(title.fg_color, title.bg_color);
        let max_x = self.win.get_max_x().max(1);
        let text_len = title.text.chars().count() as i32;
        let col = match title.alignment {
            TitleAlignment::Left => 0,
            TitleAlignment::Right => (max_x - text_len).max(0),
            TitleAlignment::Center => ((max_x - text_len) / 2).max(0),
        };
        self.win.attron(COLOR_PAIR(pair as u64));
        self.win.mv(0, col);
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
        let y = location.y as i32;
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
        let rows = text_wrap::display_row_count(&text, width) as i32;

        self.fill_solid(
            location.y as i32,
            location.x as i32,
            width as i32,
            rows,
            base_pair,
        );

        let state = TextInputState {
            text: text.clone(),
            cursor,
            selection_anchor,
        };
        let selection = text_input::selection_range(&state);
        let highlight_cells = text_wrap::selection_highlight_cells(&text, selection, width);

        let mut char_idx = 0usize;
        let mut drawn = std::collections::BTreeSet::new();
        for ch in text.chars() {
            let (line, col) = text_wrap::cursor_display_position(&text, char_idx, width);
            if ch != '\n' {
                let pair = if highlight_cells.contains(&(line, col)) {
                    selection_pair
                } else {
                    base_pair
                };
                self.win.attron(COLOR_PAIR(pair as u64));
                self.win
                    .mv(location.y as i32 + line as i32, location.x as i32 + col as i32);
                self.win.addch(ch);
                self.win.attroff(COLOR_PAIR(pair as u64));
                drawn.insert((line, col));
            }
            char_idx += 1;
        }

        for (line, col) in highlight_cells {
            if drawn.contains(&(line, col)) {
                continue;
            }
            self.win.attron(COLOR_PAIR(selection_pair as u64));
            self.win
                .mv(location.y as i32 + line as i32, location.x as i32 + col as i32);
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
        let y = location.y as i32;
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
            let row_cols =
                terminal_bounds::cols_for_printing(x, max_x, row_y, max_y) as usize;
            self.win.mv(row_y, x);
            let line =
                terminal_bounds::clip_str_to_cols(&lines[line_idx], row_cols.min(width));
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
        self.win.attron(COLOR_PAIR(pair as u64));
        for row in 0..h {
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

        let (line, col) =
            text_wrap::cursor_display_position(&input.field.text, input.cursor, input.field.width.max(1));
        let _ = curs_set(1);
        self.win.mv(
            input.location.y as i32 + line as i32,
            input.location.x as i32 + col as i32,
        );
    }

    fn handle_key(&mut self, key: TerminalKey) -> UiEvent {
        if self.handle_display_scroll(key) {
            return UiEvent::None;
        }

        if self.handle_text_input_editing(key) {
            return UiEvent::None;
        }

        match key {
            TerminalKey::Quit | TerminalKey::Escape => UiEvent::Quit,
            TerminalKey::Up | TerminalKey::Left { .. } => {
                self.focus_prev();
                UiEvent::None
            }
            TerminalKey::Down | TerminalKey::Right { .. } => {
                self.focus_next();
                UiEvent::None
            }
            TerminalKey::Enter | TerminalKey::Space => self.activate_button_on_focus(),
            _ => UiEvent::None,
        }
    }

    fn handle_display_scroll(&mut self, key: TerminalKey) -> bool {
        let Some(id) = self.current_focused_id() else {
            return false;
        };
        let Some(RuntimeElement::TextDisplay(display)) = self.element_mut_by_id(&id) else {
            return false;
        };

        match key {
            TerminalKey::AltUp if display.scroll > 0 => {
                display.scroll = scroll_view::scroll_line_up(display.scroll);
                true
            }
            TerminalKey::AltDown => {
                let total = text_wrap::wrapped_line_count(&display.display.text, display.width.max(1));
                display.scroll =
                    scroll_view::scroll_line_down(display.scroll, total, display.height.max(1));
                true
            }
            _ => false,
        }
    }

    fn handle_text_input_editing(&mut self, key: TerminalKey) -> bool {
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
                    self.apply_text_input_state(&id, text_input::paste_text(&state, &paste));
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

    fn activate_button_on_focus(&mut self) -> UiEvent {
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

    fn focus_next(&mut self) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        self.focused_position = (self.focused_position + 1) % order.len();
        self.sync_focus_flags();
    }

    fn focus_prev(&mut self) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        if self.focused_position == 0 {
            self.focused_position = order.len() - 1;
        } else {
            self.focused_position -= 1;
        }
        self.sync_focus_flags();
    }

    fn focus_order(&self) -> Vec<String> {
        let mut order = self
            .elements
            .iter()
            .map(|element| match element {
                RuntimeElement::Button(button) => (button.focus_index, button.id.clone()),
                RuntimeElement::TextInput(input) => (input.focus_index, input.id.clone()),
                RuntimeElement::TextDisplay(display) => (display.focus_index, display.id.clone()),
            })
            .collect::<Vec<_>>();
        order.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        order.into_iter().map(|(_, id)| id).collect()
    }

    fn current_focused_id(&self) -> Option<String> {
        let order = self.focus_order();
        order.get(self.focused_position).cloned()
    }

    fn sync_focus_position(&mut self) {
        let order = self.focus_order();
        if order.is_empty() {
            self.focused_position = 0;
        } else if self.focused_position >= order.len() {
            self.focused_position = order.len() - 1;
        }
        self.sync_focus_flags();
    }

    fn sync_focus_flags(&mut self) {
        let focused = self.current_focused_id();
        for element in &mut self.elements {
            let element_id = match element {
                RuntimeElement::Button(button) => button.id.as_str(),
                RuntimeElement::TextInput(input) => input.id.as_str(),
                RuntimeElement::TextDisplay(display) => display.id.as_str(),
            };
            let is_focused = focused.as_deref() == Some(element_id);
            match element {
                RuntimeElement::Button(button) => button.button.focused = is_focused,
                RuntimeElement::TextInput(input) => input.field.focused = is_focused,
                RuntimeElement::TextDisplay(display) => display.display.focused = is_focused,
            }
        }
    }

    fn text_input_state(&self, id: &str) -> TextInputState {
        match self.element_by_id(id) {
            Some(RuntimeElement::TextInput(input)) => TextInputState {
                text: input.field.text.clone(),
                cursor: input.cursor,
                selection_anchor: input.selection_anchor,
            },
            _ => TextInputState {
                text: String::new(),
                cursor: 0,
                selection_anchor: None,
            },
        }
    }

    fn apply_text_input_state(&mut self, id: &str, state: Option<TextInputState>) {
        if let Some(state) = state {
            self.set_text_input_state(id, state);
        }
    }

    fn set_text_input_state(&mut self, id: &str, state: TextInputState) {
        if let Some(RuntimeElement::TextInput(input)) = self.element_mut_by_id(id) {
            input.field.text = state.text;
            input.cursor = state.cursor;
            input.selection_anchor = state.selection_anchor;
        }
    }

    fn idx_of(&self, id: &str) -> Option<usize> {
        self.elements.iter().position(|element| match element {
            RuntimeElement::Button(button) => button.id == id,
            RuntimeElement::TextInput(input) => input.id == id,
            RuntimeElement::TextDisplay(display) => display.id == id,
        })
    }

    fn element_by_id(&self, id: &str) -> Option<&RuntimeElement> {
        self.idx_of(id).and_then(|idx| self.elements.get(idx))
    }

    fn element_mut_by_id(&mut self, id: &str) -> Option<&mut RuntimeElement> {
        self.idx_of(id).and_then(|idx| self.elements.get_mut(idx))
    }

    fn color_pair(&mut self, fg: Color, bg: Color) -> i16 {
        let fg_code = nearest_terminal_color(fg);
        let bg_code = nearest_terminal_color(bg);
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

    fn auto_reflow_for_dynamic_heights(&mut self) {
        let text_input_ids = self
            .elements
            .iter()
            .filter_map(|element| match element {
                RuntimeElement::TextInput(input) => Some(input.id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        for id in text_input_ids {
            let old_height = self.cached_heights.get(&id).copied().unwrap_or(1);
            let new_height = self.element_render_height_by_id(&id).unwrap_or(old_height);
            let delta = new_height as i32 - old_height as i32;
            if delta == 0 {
                continue;
            }
            if let Some(location) = self.element_location(&id) {
                let min_y = location.y.saturating_add(old_height as u16);
                self.shift_elements_from_min_y(&id, min_y, delta);
            }
        }
        self.refresh_height_cache();
    }

    /// Shifts every element at `y >= min_y` (except `source_id`) by `delta` rows.
    fn shift_elements_from_min_y(&mut self, source_id: &str, min_y: u16, delta: i32) {
        if delta == 0 {
            return;
        }
        for element in &mut self.elements {
            let element_id = match element {
                RuntimeElement::Button(button) => button.id.as_str(),
                RuntimeElement::TextInput(input) => input.id.as_str(),
                RuntimeElement::TextDisplay(display) => display.id.as_str(),
            };
            if element_id == source_id {
                continue;
            }
            let current_y = match element {
                RuntimeElement::Button(button) => button.button.location.y,
                RuntimeElement::TextInput(input) => input.location.y,
                RuntimeElement::TextDisplay(display) => display.location.y,
            };
            if current_y < min_y {
                continue;
            }
            let shifted = (current_y as i32 + delta).max(0) as u16;
            match element {
                RuntimeElement::Button(button) => button.button.location.y = shifted,
                RuntimeElement::TextInput(input) => input.location.y = shifted,
                RuntimeElement::TextDisplay(display) => display.location.y = shifted,
            }
        }
    }

    fn element_render_height_by_id(&self, id: &str) -> Option<usize> {
        self.element_by_id(id).map(Self::element_render_height)
    }

    fn element_render_height(element: &RuntimeElement) -> usize {
        match element {
            RuntimeElement::Button(_) => 1,
            RuntimeElement::TextInput(input) => {
                text_wrap::display_row_count(&input.field.text, input.field.width.max(1))
            }
            RuntimeElement::TextDisplay(display) => display.height.max(1),
        }
    }

    fn refresh_height_cache(&mut self) {
        self.cached_heights.clear();
        for element in &self.elements {
            let id = match element {
                RuntimeElement::Button(button) => button.id.clone(),
                RuntimeElement::TextInput(input) => input.id.clone(),
                RuntimeElement::TextDisplay(display) => display.id.clone(),
            };
            self.cached_heights
                .insert(id, Self::element_render_height(element));
        }
    }
}

impl Drop for RuntimeUi {
    fn drop(&mut self) {
        let _ = curs_set(1);
        endwin();
        let _ = terminal_input::leave_raw_mode();
    }
}

fn nearest_terminal_color(color: Color) -> i16 {
    const BASE: [(i16, (u8, u8, u8)); 8] = [
        (pancurses::COLOR_BLACK, (0, 0, 0)),
        (pancurses::COLOR_RED, (205, 49, 49)),
        (pancurses::COLOR_GREEN, (13, 188, 121)),
        (pancurses::COLOR_YELLOW, (229, 229, 16)),
        (pancurses::COLOR_BLUE, (36, 114, 200)),
        (pancurses::COLOR_MAGENTA, (188, 63, 188)),
        (pancurses::COLOR_CYAN, (17, 168, 205)),
        (pancurses::COLOR_WHITE, (229, 229, 229)),
    ];

    let mut best = BASE[0].0;
    let mut best_distance = u32::MAX;
    for (code, (r, g, b)) in BASE {
        let dr = color.r as i32 - r as i32;
        let dg = color.g as i32 - g as i32;
        let db = color.b as i32 - b as i32;
        let distance = (dr * dr + dg * dg + db * db) as u32;
        if distance < best_distance {
            best_distance = distance;
            best = code;
        }
    }
    best
}
