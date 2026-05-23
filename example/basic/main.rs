use pancurses::{curs_set, endwin, initscr, noecho, Window, COLOR_PAIR};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tersse::clipboard;
use tersse::pure::scroll_view;
use tersse::pure::text_input::{self, TextInputState};
use tersse::pure::text_wrap;
use tersse::terminal_input::{self, TerminalKey};
use tersse::{
    change_bg_color_of_focused_locked_text_input_field_elements_and_text_display_elements,
    change_bg_color_of_focused_non_locked_text_input_field_elements,
    change_bg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements,
    change_bg_color_of_non_focused_non_locked_text_input_field_elements,
    change_fg_color_of_focused_locked_text_input_field_elements_and_text_display_elements,
    change_fg_color_of_focused_non_locked_text_input_field_elements,
    change_fg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements,
    change_fg_color_of_non_focused_non_locked_text_input_field_elements, create_button,
    create_text_display_element, create_text_input_field_element, delete_tui_element,
    force_focus_on_element, read_text_from_text_input_field, set_text_input_field_lock_status,
    set_title_of_current_screen, update_text_of_text_display_element, Button, Color, Element,
    Location, TextDisplayElement, TextInputField, TitleAlignment,
};

const COL_START: i32 = 0;
const TITLE_ROW: i32 = 0;
const FIRST_CONTENT_ROW: i32 = 2;
const POLL_TIMEOUT_MS: i32 = 50;
const FLASH_FOO: Duration = Duration::from_secs(2);
const FLASH_BAR: Duration = Duration::from_secs(5);
const INPUT_WIDTH: usize = 20;
const PRESS_ME_LABEL: &str = "Press Me !!";
const CLEAR_LABEL: &str = "Clear Result";
const BUTTON_MAX_WIDTH: usize = 32;
const PAIR_DEFAULT: u64 = 0;
const PAIR_BLUE: u64 = 1;
const PAIR_CYAN: u64 = 2;
const PAIR_TITLE: u64 = 3;
const PAIR_INPUT_UNFOCUSED: u64 = 4;
const PAIR_INPUT_FOCUSED: u64 = 5;
const PAIR_INPUT_LOCKED_UNFOCUSED: u64 = 6;
const PAIR_INPUT_LOCKED_FOCUSED: u64 = 7;
const PAIR_DISPLAY_UNFOCUSED: u64 = 8;
const PAIR_DISPLAY_FOCUSED: u64 = 9;
const PAIR_INPUT_SELECT: u64 = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ElementId {
    Foo,
    Bar,
    Input,
    PressMe,
    ClearResult,
    ResultDisplay,
}

#[derive(Clone, Copy)]
struct FlashMessage {
    text: &'static str,
    expires_at: Instant,
}

#[derive(Clone, Copy, Debug)]
struct DisplayViewport {
    start_y: i32,
    start_x: i32,
    width: usize,
    height: usize,
}

struct App {
    title: tersse::ScreenTitle,
    elements: Vec<Element>,
    focus_order: Vec<ElementId>,
    focused_idx: usize,
    flash: Option<FlashMessage>,
    input_cursor: usize,
    input_selection_anchor: Option<usize>,
    display_scroll: usize,
    display_initial_height: usize,
    display_start_row: Option<i32>,
}

fn main() {
    let _ = terminal_input::enter_raw_mode();

    let win = initscr();
    noecho();
    let _ = curs_set(0);
    init_colors();

    let mut app = build_app();
    sync_focus_state(&mut app);

    loop {
        app.tick_flash();
        app.draw(&win);

        match terminal_input::poll_key(Duration::from_millis(POLL_TIMEOUT_MS as u64)) {
            Ok(Some(key)) => {
                if handle_key(&mut app, &win, key) {
                    break;
                }
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }

    let _ = curs_set(1);
    endwin();
    let _ = terminal_input::leave_raw_mode();
}

fn build_app() -> App {
    let title = set_title_of_current_screen(
        "Hello world",
        TitleAlignment::Left,
        Color::rgb(255, 255, 255),
        Color::rgb(0, 0, 0),
    );

    let foo = Element::Button(create_button(
        Location { x: 0, y: 0 },
        "Foo",
        BUTTON_MAX_WIDTH,
        Color::rgb(0, 0, 255),
        Color::rgb(255, 255, 255),
        Box::new(|| {}),
    ));
    let bar = Element::Button(create_button(
        Location { x: 0, y: 0 },
        "Bar",
        BUTTON_MAX_WIDTH,
        Color::rgb(0, 0, 255),
        Color::rgb(255, 255, 255),
        Box::new(|| {}),
    ));
    let input = Element::TextInputField(create_text_input_field_element(INPUT_WIDTH));
    let press_me = Element::Button(create_button(
        Location { x: 0, y: 0 },
        PRESS_ME_LABEL,
        BUTTON_MAX_WIDTH,
        Color::rgb(0, 0, 255),
        Color::rgb(255, 255, 255),
        Box::new(|| {}),
    ));

    let mut app = App {
        title,
        elements: vec![foo, bar, input, press_me],
        focus_order: vec![ElementId::Foo, ElementId::Bar, ElementId::Input, ElementId::PressMe],
        focused_idx: 0,
        flash: None,
        input_cursor: 0,
        input_selection_anchor: None,
        display_scroll: 0,
        display_initial_height: 0,
        display_start_row: None,
    };
    apply_input_and_display_colors(&mut app.elements);
    app
}

impl App {
    fn tick_flash(&mut self) {
        if let Some(msg) = self.flash {
            if Instant::now() >= msg.expires_at {
                self.flash = None;
            }
        }
    }

    fn draw(&mut self, win: &Window) {
        win.erase();
        draw_title(win, &self.title);
        self.relayout();

        let mut row = FIRST_CONTENT_ROW;
        self.draw_button_by_id(win, ElementId::Foo, row);
        row += 1;
        if let Some(msg) = self.flash {
            win.mv(row, COL_START);
            win.addstr(msg.text);
            row += 1;
        }
        self.draw_button_by_id(win, ElementId::Bar, row);
        row += 1;

        let input_row = row;
        if let Some(input_idx) = self.idx_of(ElementId::Input) {
            self.draw_input(win, input_idx, row);
            row += input_display_height(self.text_input_ref_by_idx(input_idx)) as i32;
        }

        if let Some(press_idx) = self.idx_of(ElementId::PressMe) {
            self.draw_button(win, press_idx, row, COL_START);
            if let Some(clear_idx) = self.idx_of(ElementId::ClearResult) {
                let press_width = button_width(self.button_ref_by_idx(press_idx));
                self.draw_button(win, clear_idx, row, COL_START + press_width as i32 + 1);
            }
            row += 1;
        }

        if let Some(display_idx) = self.idx_of(ElementId::ResultDisplay) {
            if self.display_initial_height == 0 {
                let (max_y, _) = win.get_max_yx();
                self.display_initial_height = (max_y - row).max(1) as usize;
            }
            self.display_start_row = Some(row);
            self.sync_display_scroll(win, row);
            self.draw_display(win, display_idx, row);
        } else {
            self.display_start_row = None;
        }

        if self.input_editing() {
            if let Some(input_idx) = self.idx_of(ElementId::Input) {
                let field = self.text_input_ref_by_idx(input_idx);
                let (line, col) =
                    text_wrap::cursor_display_position(&field.text, self.input_cursor, field.width);
                curs_set(1);
                win.mv(input_row + line as i32, COL_START + col as i32);
            }
        } else {
            curs_set(0);
        }

        win.refresh();
    }

    fn draw_button_by_id(&self, win: &Window, id: ElementId, row: i32) {
        if let Some(idx) = self.idx_of(id) {
            self.draw_button(win, idx, row, COL_START);
        }
    }

    fn draw_button(&self, win: &Window, element_idx: usize, row: i32, col: i32) {
        let button = self.button_ref_by_idx(element_idx);
        let pair = if button.focused { PAIR_CYAN } else { PAIR_BLUE };
        win.attron(COLOR_PAIR(pair));
        win.mv(row, col);
        win.addstr(&button.display_string);
        win.attroff(COLOR_PAIR(pair));
    }

    fn draw_input(&self, win: &Window, input_idx: usize, row: i32) {
        let field = self.text_input_ref_by_idx(input_idx);
        let pair = input_color_pair(field);
        let width = field.width;
        let rows = input_display_height(field) as i32;
        fill_solid(win, row, COL_START, width as i32, rows, pair);

        let input_state = TextInputState {
            text: field.text.clone(),
            cursor: self.input_cursor,
            selection_anchor: self.input_selection_anchor,
        };
        let selection = text_input::selection_range(&input_state);
        let highlight_cells = text_wrap::selection_highlight_cells(&field.text, selection, width);

        let mut char_index = 0usize;
        let mut drawn_cells = BTreeSet::new();
        for ch in field.text.chars() {
            let (line, col) = text_wrap::cursor_display_position(&field.text, char_index, width);
            if ch != '\n' {
                let selected = highlight_cells.contains(&(line, col));
                let ch_pair = if selected { PAIR_INPUT_SELECT } else { pair };
                win.attron(COLOR_PAIR(ch_pair));
                win.mv(row + line as i32, COL_START + col as i32);
                win.addch(ch);
                win.attroff(COLOR_PAIR(ch_pair));
                drawn_cells.insert((line, col));
            }
            char_index += 1;
        }

        for (line, col) in highlight_cells {
            if drawn_cells.contains(&(line, col)) {
                continue;
            }
            win.attron(COLOR_PAIR(PAIR_INPUT_SELECT));
            win.mv(row + line as i32, COL_START + col as i32);
            win.addch(' ');
            win.attroff(COLOR_PAIR(PAIR_INPUT_SELECT));
        }
    }

    fn draw_display(&self, win: &Window, display_idx: usize, row: i32) {
        let display = self.text_display_ref_by_idx(display_idx);
        let viewport = display_viewport(win, row, self.display_initial_height);
        let pair = if display.focused {
            PAIR_DISPLAY_FOCUSED
        } else {
            PAIR_DISPLAY_UNFOCUSED
        };

        if viewport.height == 0 {
            return;
        }

        fill_solid(
            win,
            viewport.start_y,
            viewport.start_x,
            viewport.width as i32,
            viewport.height as i32,
            pair,
        );

        let lines = text_wrap::wrapped_lines(&display.text, viewport.width);
        if lines.is_empty() {
            return;
        }

        let total = lines.len();
        let offset = scroll_view::clamp_scroll_offset(self.display_scroll, total, viewport.height);
        let range = scroll_view::visible_line_range(offset, viewport.height, total);

        win.attron(COLOR_PAIR(pair));
        for (viewport_row, line_idx) in range.enumerate() {
            win.mv(viewport.start_y + viewport_row as i32, viewport.start_x);
            let line = lines[line_idx]
                .chars()
                .take(viewport.width)
                .collect::<String>();
            win.addstr(&line);
        }
        win.attroff(COLOR_PAIR(pair));
    }

    fn activate_focused(&mut self) {
        let Some(id) = self.current_focused_id() else {
            return;
        };
        match id {
            ElementId::Foo => {
                self.flash = Some(FlashMessage {
                    text: "Button 1",
                    expires_at: Instant::now() + FLASH_FOO,
                });
            }
            ElementId::Bar => {
                self.flash = Some(FlashMessage {
                    text: "Button 2",
                    expires_at: Instant::now() + FLASH_BAR,
                });
            }
            ElementId::PressMe => self.press_me_action(),
            ElementId::ClearResult => self.clear_result_action(),
            ElementId::Input | ElementId::ResultDisplay => {}
        }
        self.sync_focus_after_structure_change();
    }

    fn press_me_action(&mut self) {
        let Some(input_idx) = self.idx_of(ElementId::Input) else {
            return;
        };
        let input_value = read_text_from_text_input_field(self.text_input_ref_by_idx(input_idx));
        let reversed = input_value.chars().rev().collect::<String>();
        let display_text = reversed.repeat(10);
        self.display_scroll = 0;
        set_text_input_field_lock_status(self.text_input_mut_by_idx(input_idx), true);
        self.input_selection_anchor = None;

        if self.idx_of(ElementId::ClearResult).is_none() {
            self.elements.push(Element::Button(create_button(
                Location { x: 0, y: 0 },
                CLEAR_LABEL,
                BUTTON_MAX_WIDTH,
                Color::rgb(0, 0, 255),
                Color::rgb(255, 255, 255),
                Box::new(|| {}),
            )));
        }

        if let Some(display_idx) = self.idx_of(ElementId::ResultDisplay) {
            let display = self.text_display_mut_by_idx(display_idx);
            update_text_of_text_display_element(display, display_text.clone());
        } else {
            self.elements
                .push(Element::TextDisplayElement(create_text_display_element(display_text)));
        }

        self.normalize_result_focus_order();
        apply_input_and_display_colors(&mut self.elements);
    }

    fn normalize_result_focus_order(&mut self) {
        let has_clear = self.idx_of(ElementId::ClearResult).is_some();
        let has_display = self.idx_of(ElementId::ResultDisplay).is_some();
        if !has_clear && !has_display {
            return;
        }

        self.focus_order
            .retain(|id| *id != ElementId::ClearResult && *id != ElementId::ResultDisplay);

        let insert_at = self
            .focus_order
            .iter()
            .position(|id| *id == ElementId::PressMe)
            .map(|idx| idx + 1)
            .unwrap_or(self.focus_order.len());

        if has_clear {
            self.focus_order.insert(insert_at, ElementId::ClearResult);
        }
        if has_display {
            let display_at = if has_clear { insert_at + 1 } else { insert_at };
            self.focus_order.insert(display_at, ElementId::ResultDisplay);
        }
    }

    fn clear_result_action(&mut self) {
        self.delete_by_id(ElementId::ResultDisplay);
        self.delete_by_id(ElementId::ClearResult);
        self.display_scroll = 0;
        self.display_initial_height = 0;
        self.display_start_row = None;
        if let Some(input_idx) = self.idx_of(ElementId::Input) {
            set_text_input_field_lock_status(self.text_input_mut_by_idx(input_idx), false);
        }
        self.input_selection_anchor = None;
        apply_input_and_display_colors(&mut self.elements);
    }

    fn delete_by_id(&mut self, id: ElementId) {
        if let Some(idx) = self.idx_of(id) {
            let _ = delete_tui_element(&mut self.elements, idx);
            if let Some(order_idx) = self.focus_order.iter().position(|item| *item == id) {
                self.focus_order.remove(order_idx);
                if self.focused_idx >= self.focus_order.len() && !self.focus_order.is_empty() {
                    self.focused_idx = self.focus_order.len() - 1;
                } else if self.focus_order.is_empty() {
                    self.focused_idx = 0;
                }
            }
        }
    }

    fn focus_next(&mut self) {
        if self.focus_order.is_empty() {
            return;
        }
        self.focused_idx = (self.focused_idx + 1) % self.focus_order.len();
        sync_focus_state(self);
    }

    fn focus_prev(&mut self) {
        if self.focus_order.is_empty() {
            return;
        }
        if self.focused_idx == 0 {
            self.focused_idx = self.focus_order.len().saturating_sub(1);
        } else {
            self.focused_idx -= 1;
        }
        sync_focus_state(self);
    }

    fn input_editing(&self) -> bool {
        self.current_focused_id() == Some(ElementId::Input)
            && self
                .idx_of(ElementId::Input)
                .map(|idx| !self.text_input_ref_by_idx(idx).locked)
                .unwrap_or(false)
    }

    fn input_focused(&self) -> bool {
        self.current_focused_id() == Some(ElementId::Input)
    }

    fn input_focused_locked(&self) -> bool {
        self.input_focused()
            && self
                .idx_of(ElementId::Input)
                .map(|idx| self.text_input_ref_by_idx(idx).locked)
                .unwrap_or(false)
    }

    fn input_state(&self) -> TextInputState {
        let idx = self.idx_of(ElementId::Input).expect("input element exists");
        TextInputState {
            text: self.text_input_ref_by_idx(idx).text.clone(),
            cursor: self.input_cursor,
            selection_anchor: self.input_selection_anchor,
        }
    }

    fn set_input_state(&mut self, state: TextInputState) {
        let idx = self.idx_of(ElementId::Input).expect("input element exists");
        self.text_input_mut_by_idx(idx).text = state.text;
        self.input_cursor = state.cursor;
        self.input_selection_anchor = state.selection_anchor;
    }

    fn apply_input(&mut self, next: Option<TextInputState>) {
        if self.input_editing() {
            if let Some(state) = next {
                self.set_input_state(state);
            }
        }
    }

    fn handle_copy(&mut self) {
        if let Some((state, text)) = text_input::copy_selection(&self.input_state()) {
            if clipboard::set_text(&text) {
                self.set_input_state(state);
            }
        }
    }

    fn handle_cut(&mut self) {
        if let Some((state, text)) = text_input::cut_selection(&self.input_state()) {
            if clipboard::set_text(&text) {
                self.set_input_state(state);
            }
        }
    }

    fn handle_paste(&mut self) {
        if let Some(paste) = clipboard::get_text() {
            self.apply_input(text_input::paste_text(&self.input_state(), &paste));
        }
    }

    fn current_focused_id(&self) -> Option<ElementId> {
        self.focus_order.get(self.focused_idx).copied()
    }

    fn idx_of(&self, id: ElementId) -> Option<usize> {
        self.elements.iter().position(|element| id_for_element(element) == id)
    }

    fn button_ref_by_idx(&self, idx: usize) -> &Button {
        match &self.elements[idx] {
            Element::Button(button) => button,
            _ => panic!("element at index is not a button"),
        }
    }

    fn text_input_ref_by_idx(&self, idx: usize) -> &TextInputField {
        match &self.elements[idx] {
            Element::TextInputField(field) => field,
            _ => panic!("element at index is not a text input"),
        }
    }

    fn text_input_mut_by_idx(&mut self, idx: usize) -> &mut TextInputField {
        match &mut self.elements[idx] {
            Element::TextInputField(field) => field,
            _ => panic!("element at index is not a text input"),
        }
    }

    fn text_display_ref_by_idx(&self, idx: usize) -> &TextDisplayElement {
        match &self.elements[idx] {
            Element::TextDisplayElement(display) => display,
            _ => panic!("element at index is not a text display"),
        }
    }

    fn text_display_mut_by_idx(&mut self, idx: usize) -> &mut TextDisplayElement {
        match &mut self.elements[idx] {
            Element::TextDisplayElement(display) => display,
            _ => panic!("element at index is not a text display"),
        }
    }

    fn relayout(&mut self) {
        let mut next_row = FIRST_CONTENT_ROW as u16;
        for id in [ElementId::Foo, ElementId::Bar, ElementId::Input, ElementId::PressMe] {
            if let Some(idx) = self.idx_of(id) {
                assign_location(&mut self.elements[idx], Location { x: 0, y: next_row });
                next_row = next_row.saturating_add(1);
                if id == ElementId::Foo && self.flash.is_some() {
                    next_row = next_row.saturating_add(1);
                }
            }
        }
    }

    fn display_focused(&self) -> bool {
        self.current_focused_id() == Some(ElementId::ResultDisplay)
    }

    fn display_viewport(&self, win: &Window) -> Option<DisplayViewport> {
        let row = self.display_start_row?;
        Some(display_viewport(win, row, self.display_initial_height))
    }

    fn display_line_count(&self, win: &Window) -> usize {
        let Some(idx) = self.idx_of(ElementId::ResultDisplay) else {
            return 0;
        };
        let Some(viewport) = self.display_viewport(win) else {
            return 0;
        };
        text_wrap::wrapped_line_count(
            &self.text_display_ref_by_idx(idx).text,
            viewport.width,
        )
    }

    fn display_content_overflows(&self, win: &Window) -> bool {
        let Some(viewport) = self.display_viewport(win) else {
            return false;
        };
        scroll_view::content_overflows(self.display_line_count(win), viewport.height)
    }

    fn display_scrollable(&self, win: &Window) -> bool {
        self.display_focused() && self.display_content_overflows(win)
    }

    fn sync_display_scroll(&mut self, win: &Window, start_y: i32) {
        let Some(idx) = self.idx_of(ElementId::ResultDisplay) else {
            return;
        };
        let viewport = display_viewport(win, start_y, self.display_initial_height);
        let total = text_wrap::wrapped_line_count(
            &self.text_display_ref_by_idx(idx).text,
            viewport.width,
        );
        self.display_scroll =
            scroll_view::clamp_scroll_offset(self.display_scroll, total, viewport.height);
    }

    fn scroll_display_up(&mut self) {
        self.display_scroll = scroll_view::scroll_line_up(self.display_scroll);
    }

    fn scroll_display_down(&mut self, win: &Window) {
        let Some(viewport) = self.display_viewport(win) else {
            return;
        };
        let total = self.display_line_count(win);
        self.display_scroll =
            scroll_view::scroll_line_down(self.display_scroll, total, viewport.height);
    }

    fn sync_focus_after_structure_change(&mut self) {
        if self.focus_order.is_empty() {
            return;
        }
        if self.focused_idx >= self.focus_order.len() {
            self.focused_idx = self.focus_order.len() - 1;
        }
        sync_focus_state(self);
    }
}

fn input_display_height(field: &TextInputField) -> usize {
    text_wrap::display_row_count(&field.text, field.width)
}

fn display_viewport(win: &Window, start_y: i32, initial_height: usize) -> DisplayViewport {
    let (max_y, max_x) = win.get_max_yx();
    let remaining = (max_y - start_y).max(0) as usize;
    let height = if initial_height == 0 {
        remaining
    } else {
        initial_height.min(remaining)
    };
    DisplayViewport {
        start_y,
        start_x: COL_START,
        width: (max_x - COL_START).max(1) as usize,
        height,
    }
}

fn handle_key(app: &mut App, win: &Window, key: TerminalKey) -> bool {
    if app.display_focused() {
        match key {
            TerminalKey::AltUp if app.display_scroll > 0 => {
                app.scroll_display_up();
                return false;
            }
            TerminalKey::AltDown if app.display_scrollable(win) => {
                app.scroll_display_down(win);
                return false;
            }
            _ => {}
        }
    }

    if (app.input_editing() || app.input_focused_locked())
        && matches!(key, TerminalKey::Up | TerminalKey::Down)
    {
        match key {
            TerminalKey::Up => app.focus_prev(),
            TerminalKey::Down => app.focus_next(),
            _ => {}
        }
        return false;
    }

    if app.input_editing() {
        handle_input_editing_key(app, key);
        return false;
    }

    match key {
        TerminalKey::Quit => true,
        TerminalKey::Up | TerminalKey::Left { .. } => {
            app.focus_prev();
            false
        }
        TerminalKey::Down | TerminalKey::Right { .. } => {
            app.focus_next();
            false
        }
        TerminalKey::Enter | TerminalKey::Space => {
            app.activate_focused();
            false
        }
        _ => false,
    }
}

fn handle_input_editing_key(app: &mut App, key: TerminalKey) {
    let state = app.input_state();
    match key {
        TerminalKey::Left { extend_selection } => {
            app.apply_input(text_input::cursor_left(&state, extend_selection));
        }
        TerminalKey::Right { extend_selection } => {
            app.apply_input(text_input::cursor_right(&state, extend_selection));
        }
        TerminalKey::Backspace => app.apply_input(text_input::backspace(&state)),
        TerminalKey::Delete => app.apply_input(text_input::delete_forward(&state)),
        TerminalKey::Enter => app.apply_input(text_input::insert_newline(&state)),
        TerminalKey::Space => app.apply_input(text_input::insert_char(&state, ' ')),
        TerminalKey::Tab => app.apply_input(text_input::insert_tab(&state)),
        TerminalKey::Copy => app.handle_copy(),
        TerminalKey::Cut => app.handle_cut(),
        TerminalKey::Paste => app.handle_paste(),
        TerminalKey::Quit => app.apply_input(text_input::insert_char(&state, 'q')),
        TerminalKey::Char(c) if c == '\t' => app.apply_input(text_input::insert_tab(&state)),
        TerminalKey::Char(c) if !c.is_control() => app.apply_input(text_input::insert_char(&state, c)),
        _ => {}
    }
}

fn fill_solid(win: &Window, y: i32, x: i32, w: i32, h: i32, pair: u64) {
    win.attron(COLOR_PAIR(pair));
    for row in 0..h {
        win.mv(y + row, x);
        for _ in 0..w {
            win.addch(' ');
        }
    }
    win.attroff(COLOR_PAIR(pair));
}

fn draw_title(win: &Window, title: &tersse::ScreenTitle) {
    let max_x = win.get_max_x().max(1);
    let text_len = title.text.chars().count() as i32;
    let col = match title.alignment {
        TitleAlignment::Left => COL_START,
        TitleAlignment::Right => (max_x - text_len).max(0),
        TitleAlignment::Center => ((max_x - text_len) / 2).max(0),
    };
    win.attron(COLOR_PAIR(PAIR_TITLE));
    win.mv(TITLE_ROW, col);
    win.addstr(&title.text);
    win.attroff(COLOR_PAIR(PAIR_TITLE));
}

fn sync_focus_state(app: &mut App) {
    if app.focus_order.is_empty() {
        return;
    }
    let Some(id) = app.current_focused_id() else {
        return;
    };
    if let Some(idx) = app.idx_of(id) {
        let _ = force_focus_on_element(&mut app.elements, idx);
    }
}

fn id_for_element(element: &Element) -> ElementId {
    match element {
        Element::Button(button) if button.display_string == "Foo" => ElementId::Foo,
        Element::Button(button) if button.display_string == "Bar" => ElementId::Bar,
        Element::Button(button) if button.display_string == PRESS_ME_LABEL => ElementId::PressMe,
        Element::Button(button) if button.display_string == CLEAR_LABEL => ElementId::ClearResult,
        Element::TextInputField(_) => ElementId::Input,
        Element::TextDisplayElement(_) => ElementId::ResultDisplay,
        Element::Button(_) => panic!("unexpected button label"),
    }
}

fn apply_input_and_display_colors(elements: &mut [Element]) {
    change_bg_color_of_non_focused_non_locked_text_input_field_elements(elements, Color::rgb(0, 0, 0));
    change_fg_color_of_non_focused_non_locked_text_input_field_elements(
        elements,
        Color::rgb(255, 255, 255),
    );
    change_bg_color_of_focused_non_locked_text_input_field_elements(elements, Color::rgb(255, 255, 255));
    change_fg_color_of_focused_non_locked_text_input_field_elements(elements, Color::rgb(0, 0, 0));
    change_bg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements(
        elements,
        Color::rgb(0, 0, 0),
    );
    change_fg_color_of_non_focused_locked_text_input_field_elements_and_text_display_elements(
        elements,
        Color::rgb(255, 255, 0),
    );
    change_bg_color_of_focused_locked_text_input_field_elements_and_text_display_elements(
        elements,
        Color::rgb(255, 255, 255),
    );
    change_fg_color_of_focused_locked_text_input_field_elements_and_text_display_elements(
        elements,
        Color::rgb(255, 0, 0),
    );
}

fn input_color_pair(field: &TextInputField) -> u64 {
    if field.locked {
        if field.focused {
            PAIR_INPUT_LOCKED_FOCUSED
        } else {
            PAIR_INPUT_LOCKED_UNFOCUSED
        }
    } else if field.focused {
        PAIR_INPUT_FOCUSED
    } else {
        PAIR_INPUT_UNFOCUSED
    }
}

fn button_width(button: &Button) -> usize {
    button.display_string.chars().count()
}

fn assign_location(element: &mut Element, location: Location) {
    if let Element::Button(button) = element {
        button.location = location;
    }
}

fn init_colors() {
    pancurses::start_color();
    pancurses::use_default_colors();
    pancurses::init_pair(PAIR_BLUE as i16, pancurses::COLOR_WHITE, pancurses::COLOR_BLUE);
    pancurses::init_pair(PAIR_CYAN as i16, pancurses::COLOR_BLACK, pancurses::COLOR_CYAN);
    pancurses::init_pair(PAIR_TITLE as i16, pancurses::COLOR_WHITE, pancurses::COLOR_BLACK);
    pancurses::init_pair(
        PAIR_INPUT_UNFOCUSED as i16,
        pancurses::COLOR_WHITE,
        pancurses::COLOR_BLACK,
    );
    pancurses::init_pair(
        PAIR_INPUT_FOCUSED as i16,
        pancurses::COLOR_BLACK,
        pancurses::COLOR_WHITE,
    );
    pancurses::init_pair(
        PAIR_INPUT_LOCKED_UNFOCUSED as i16,
        pancurses::COLOR_YELLOW,
        pancurses::COLOR_BLACK,
    );
    pancurses::init_pair(
        PAIR_INPUT_LOCKED_FOCUSED as i16,
        pancurses::COLOR_RED,
        pancurses::COLOR_WHITE,
    );
    pancurses::init_pair(
        PAIR_DISPLAY_UNFOCUSED as i16,
        pancurses::COLOR_YELLOW,
        pancurses::COLOR_BLACK,
    );
    pancurses::init_pair(
        PAIR_DISPLAY_FOCUSED as i16,
        pancurses::COLOR_RED,
        pancurses::COLOR_WHITE,
    );
    pancurses::init_pair(
        PAIR_INPUT_SELECT as i16,
        pancurses::COLOR_WHITE,
        pancurses::COLOR_BLACK,
    );
    pancurses::init_pair(PAIR_DEFAULT as i16, pancurses::COLOR_WHITE, pancurses::COLOR_BLACK);
}
