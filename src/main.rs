//! Minimal terminal curses UI with Gemini streaming test.

mod backend;
mod constants;
mod pure;
mod terminal_input;

use backend::{start_stream, AiStreamEvent};
use constants::{
    button_color_pair, clear_resp_btn_x, gemini_api_key, init_ui_colors, load_env_files,
    text_input_color_pair, AI_INPUT_HEIGHT, AI_INPUT_WIDTH,
    AI_MISSING_API_KEY_RES_TEXT,
    AI_NO_PROMPT_RES_TEXT, AI_RES_WAITING_TEXT, BTN_HEIGHT, BTN_WIDTH, CLEAR_RESP_BTN_HEIGHT,
    CLEAR_RESP_BTN_LABEL,
    CLEAR_RESP_BTN_WIDTH, COL_BTN, COL_FLASH_TEXT, COL_TITLE, PAIR_AI_RESPONSE,
    PAIR_TEXT_INPUT_LOCKED_NON_HOVERED, PAIR_TEXT_INPUT_SELECT, ROW_FIRST_BTN,
    ROW_TITLE, TEST_AI_BTN_HEIGHT, TEST_AI_BTN_LABEL, TEST_AI_BTN_WIDTH,
};
use pancurses::{echo, endwin, initscr, noecho};
use pancurses::{curs_set, COLOR_PAIR};
use terminal_input::TerminalKey;
use pure::text_input::{self, TextInputState};
use pure::text_wrap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

const FLASH_SECS: Duration = Duration::from_secs(2);
const POLL_MS: i32 = 50;
/// Minimum time between Gemini requests (avoids duplicate clicks / 429 bursts).
const AI_REQUEST_DEBOUNCE: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
enum DemoButton {
    Foo,
    Bar,
}

impl DemoButton {
    fn label(self) -> &'static str {
        match self {
            DemoButton::Foo => "Foo",
            DemoButton::Bar => "Bar",
        }
    }

    fn flash_text(self) -> &'static str {
        match self {
            DemoButton::Foo => "Button 1",
            DemoButton::Bar => "Button 2",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Foo,
    Bar,
    AiInput,
    TestAi,
    ClearResponse,
}

impl Focus {
    fn next(self, show_clear: bool) -> Self {
        match self {
            Focus::Foo => Focus::Bar,
            Focus::Bar => Focus::AiInput,
            Focus::AiInput => Focus::TestAi,
            Focus::TestAi => {
                if show_clear {
                    Focus::ClearResponse
                } else {
                    Focus::Foo
                }
            }
            Focus::ClearResponse => Focus::Foo,
        }
    }

    fn prev(self, show_clear: bool) -> Self {
        match self {
            Focus::Foo => {
                if show_clear {
                    Focus::ClearResponse
                } else {
                    Focus::TestAi
                }
            }
            Focus::Bar => Focus::Foo,
            Focus::AiInput => Focus::Bar,
            Focus::TestAi => Focus::AiInput,
            Focus::ClearResponse => Focus::TestAi,
        }
    }
}

#[derive(Clone, Copy)]
struct Layout {
    foo_y: i32,
    bar_y: i32,
    foo_flash_y: Option<i32>,
    bar_flash_y: Option<i32>,
    ai_input_y: i32,
    test_ai_y: i32,
    ai_response_y: i32,
    _ai_response_rows: i32,
}

fn compute_layout(
    foo_flash: bool,
    bar_flash: bool,
    ai_input_rows: i32,
    ai_response_rows: i32,
) -> Layout {
    let mut y = ROW_FIRST_BTN;

    let foo_y = y;
    y += BTN_HEIGHT;

    let foo_flash_y = if foo_flash {
        let row = y;
        y += BTN_HEIGHT;
        Some(row)
    } else {
        None
    };

    let bar_y = y;
    y += BTN_HEIGHT;

    let bar_flash_y = if bar_flash {
        let row = y;
        y += BTN_HEIGHT;
        Some(row)
    } else {
        None
    };

    let ai_input_y = y;
    y += ai_input_rows;

    let test_ai_y = y;
    y += TEST_AI_BTN_HEIGHT;

    let ai_response_y = y;

    Layout {
        foo_y,
        bar_y,
        foo_flash_y,
        bar_flash_y,
        ai_input_y,
        test_ai_y,
        ai_response_y,
        _ai_response_rows: ai_response_rows,
    }
}

struct App {
    focus: Focus,
    foo_flash: Option<Instant>,
    bar_flash: Option<Instant>,
    ai_input: String,
    ai_input_cursor: usize,
    ai_input_selection_anchor: Option<usize>,
    ai_input_locked: bool,
    ai_response: String,
    ai_streaming: bool,
    ai_rx: Option<Receiver<AiStreamEvent>>,
    show_clear_response: bool,
    last_ai_request_at: Option<Instant>,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            focus: Focus::Foo,
            foo_flash: None,
            bar_flash: None,
            ai_input: String::new(),
            ai_input_cursor: 0,
            ai_input_selection_anchor: None,
            ai_input_locked: false,
            ai_response: String::new(),
            ai_streaming: false,
            ai_rx: None,
            show_clear_response: false,
            last_ai_request_at: None,
            quit: false,
        }
    }

    /// Streaming started but no model output yet (waiting text replaces prompt).
    fn ai_waiting_for_first_token(&self) -> bool {
        self.ai_streaming && self.ai_response.is_empty()
    }

    fn layout(&self) -> Layout {
        let ai_input_rows = text_wrap::display_row_count(&self.ai_input, AI_INPUT_WIDTH as usize)
            .max(AI_INPUT_HEIGHT as usize) as i32;
        let response_rows =
            text_wrap::wrapped_line_count(&self.ai_response, text_wrap_width() as usize) as i32;
        compute_layout(
            self.foo_flash.is_some(),
            self.bar_flash.is_some(),
            ai_input_rows,
            response_rows,
        )
    }

    fn activate_demo(&mut self, button: DemoButton) {
        let now = Instant::now();
        match button {
            DemoButton::Foo => self.foo_flash = Some(now),
            DemoButton::Bar => self.bar_flash = Some(now),
        }
    }

    fn start_ai_request(&mut self) {
        if self.ai_streaming {
            return;
        }

        if let Some(at) = self.last_ai_request_at {
            if at.elapsed() < AI_REQUEST_DEBOUNCE {
                return;
            }
        }

        if self.ai_input.trim().is_empty() {
            self.ai_response = AI_NO_PROMPT_RES_TEXT.to_string();
            self.show_clear_response = true;
            self.ai_input_locked = false;
            self.ai_rx = None;
            return;
        }

        let key = match gemini_api_key() {
            Ok(key) => key,
            Err(_) => {
                self.ai_response = AI_MISSING_API_KEY_RES_TEXT.to_string();
                self.show_clear_response = true;
                self.ai_input_locked = false;
                self.ai_rx = None;
                return;
            }
        };

        self.ai_input_locked = true;
        self.ai_input_selection_anchor = None;
        self.ai_response.clear();
        self.show_clear_response = false;
        self.ai_streaming = true;
        self.last_ai_request_at = Some(Instant::now());
        self.ai_rx = Some(start_stream(&key, &self.ai_input));
    }

    fn clear_ai_response(&mut self) {
        self.ai_response.clear();
        self.show_clear_response = false;
        self.ai_input_locked = false;
        self.ai_input_selection_anchor = None;
        if self.focus == Focus::ClearResponse {
            self.focus = Focus::TestAi;
        }
    }

    fn poll_ai(&mut self) {
        let mut events = Vec::new();
        if let Some(rx) = &self.ai_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                AiStreamEvent::Token(token) => self.ai_response.push_str(&token),
                AiStreamEvent::Done => {
                    self.ai_streaming = false;
                    self.ai_rx = None;
                    if !self.ai_response.is_empty() {
                        self.show_clear_response = true;
                    }
                }
                AiStreamEvent::Error(msg) => {
                    self.ai_response = msg;
                    self.ai_streaming = false;
                    self.ai_rx = None;
                    self.show_clear_response = true;
                }
            }
        }
    }

    fn activate_focused(&mut self) {
        match self.focus {
            Focus::Foo => self.activate_demo(DemoButton::Foo),
            Focus::Bar => self.activate_demo(DemoButton::Bar),
            Focus::TestAi => self.start_ai_request(),
            Focus::ClearResponse => self.clear_ai_response(),
            Focus::AiInput => {}
        }
    }

    fn tick(&mut self) {
        tick_flash(&mut self.foo_flash);
        tick_flash(&mut self.bar_flash);
        self.poll_ai();
    }
}

fn tick_flash(slot: &mut Option<Instant>) {
    if let Some(at) = *slot {
        if at.elapsed() >= FLASH_SECS {
            *slot = None;
        }
    }
}

fn text_wrap_width() -> i32 {
    72
}

fn fill_solid(win: &pancurses::Window, y: i32, x: i32, w: i32, h: i32, pair: u64) {
    win.attron(COLOR_PAIR(pair));
    for row in 0..h {
        win.mv(y + row, x);
        for _ in 0..w {
            win.addch(' ');
        }
    }
    win.attroff(COLOR_PAIR(pair));
}

fn draw_action_button(
    win: &pancurses::Window,
    y: i32,
    x: i32,
    width: i32,
    height: i32,
    label: &str,
    focused: bool,
) {
    let pair = button_color_pair(focused);
    fill_solid(win, y, x, width, height, pair);
    win.attron(COLOR_PAIR(pair));
    win.mv(y, x);
    win.addstr(label);
    win.attroff(COLOR_PAIR(pair));
}

fn draw_demo_button(win: &pancurses::Window, y: i32, button: DemoButton, focused: bool) {
    draw_action_button(
        win,
        y,
        COL_BTN,
        BTN_WIDTH,
        BTN_HEIGHT,
        button.label(),
        focused,
    );
}

fn draw_ai_input(
    win: &pancurses::Window,
    y: i32,
    text: &str,
    cursor: usize,
    selection_anchor: Option<usize>,
    locked: bool,
    focused: bool,
) {
    let width = AI_INPUT_WIDTH as usize;
    let rows = text_wrap::display_row_count(text, width) as i32;
    let pair = text_input_color_pair(focused, locked);
    fill_solid(win, y, COL_BTN, AI_INPUT_WIDTH, rows, pair);

    let input_state = TextInputState {
        text: text.to_string(),
        cursor,
        selection_anchor,
    };
    let selection = text_input::selection_range(&input_state);

    let mut char_index = 0usize;
    for ch in text.chars() {
        let (line, col) = text_wrap::cursor_display_position(text, char_index, width);
        let selected = selection
            .map(|(start, end)| char_index >= start && char_index < end)
            .unwrap_or(false);
        let ch_pair = if selected {
            PAIR_TEXT_INPUT_SELECT
        } else {
            pair
        };
        win.attron(COLOR_PAIR(ch_pair));
        win.mv(y + line as i32, COL_BTN + col as i32);
        win.addch(ch);
        win.attroff(COLOR_PAIR(ch_pair));
        char_index += 1;
    }
}

fn draw_ai_response(win: &pancurses::Window, y: i32, text: &str) {
    win.attron(COLOR_PAIR(PAIR_AI_RESPONSE));
    for (i, line) in text_wrap::wrapped_lines(text, text_wrap_width() as usize).iter().enumerate() {
        win.mv(y + i as i32, COL_BTN);
        win.addstr(line);
    }
    win.attroff(COLOR_PAIR(PAIR_AI_RESPONSE));
}

fn draw_ai_waiting(win: &pancurses::Window, y: i32, text: &str) {
    win.attron(COLOR_PAIR(PAIR_TEXT_INPUT_LOCKED_NON_HOVERED));
    win.mv(y, COL_BTN);
    win.addstr(text);
    win.attroff(COLOR_PAIR(PAIR_TEXT_INPUT_LOCKED_NON_HOVERED));
}

fn draw_ui(win: &pancurses::Window, app: &App) {
    let layout = app.layout();

    win.erase();

    win.mv(ROW_TITLE, COL_TITLE);
    win.addstr("Hello world");

    draw_demo_button(win, layout.foo_y, DemoButton::Foo, app.focus == Focus::Foo);
    draw_demo_button(win, layout.bar_y, DemoButton::Bar, app.focus == Focus::Bar);

    if let Some(y) = layout.foo_flash_y {
        win.mv(y, COL_FLASH_TEXT);
        win.addstr(DemoButton::Foo.flash_text());
    }
    if let Some(y) = layout.bar_flash_y {
        win.mv(y, COL_FLASH_TEXT);
        win.addstr(DemoButton::Bar.flash_text());
    }

    draw_ai_input(
        win,
        layout.ai_input_y,
        &app.ai_input,
        app.ai_input_cursor,
        app.ai_input_selection_anchor,
        app.ai_input_locked,
        app.focus == Focus::AiInput,
    );

    draw_action_button(
        win,
        layout.test_ai_y,
        COL_BTN,
        TEST_AI_BTN_WIDTH,
        TEST_AI_BTN_HEIGHT,
        TEST_AI_BTN_LABEL,
        app.focus == Focus::TestAi,
    );

    if app.show_clear_response {
        draw_action_button(
            win,
            layout.test_ai_y,
            clear_resp_btn_x(),
            CLEAR_RESP_BTN_WIDTH,
            CLEAR_RESP_BTN_HEIGHT,
            CLEAR_RESP_BTN_LABEL,
            app.focus == Focus::ClearResponse,
        );
    }

    if app.ai_waiting_for_first_token() {
        draw_ai_waiting(win, layout.ai_response_y, AI_RES_WAITING_TEXT);
    } else if !app.ai_response.is_empty() {
        draw_ai_response(win, layout.ai_response_y, &app.ai_response);
    }

    if app.focus == Focus::AiInput && !app.ai_input_locked {
        let (line, col) = text_wrap::cursor_display_position(
            &app.ai_input,
            app.ai_input_cursor,
            AI_INPUT_WIDTH as usize,
        );
        curs_set(1);
        win.mv(layout.ai_input_y + line as i32, COL_BTN + col as i32);
    } else {
        curs_set(0);
    }

    win.refresh();
}

fn ai_input_editing(app: &App) -> bool {
    app.focus == Focus::AiInput && !app.ai_input_locked
}

fn ai_input_state(app: &App) -> TextInputState {
    TextInputState {
        text: app.ai_input.clone(),
        cursor: app.ai_input_cursor,
        selection_anchor: app.ai_input_selection_anchor,
    }
}

fn set_ai_input_state(app: &mut App, state: TextInputState) {
    app.ai_input = state.text;
    app.ai_input_cursor = state.cursor;
    app.ai_input_selection_anchor = state.selection_anchor;
}

fn apply_ai_input(app: &mut App, next: Option<TextInputState>) {
    if ai_input_editing(app) {
        if let Some(state) = next {
            set_ai_input_state(app, state);
        }
    }
}

fn handle_input_char(app: &mut App, c: char) {
    apply_ai_input(
        app,
        text_input::insert_char(&ai_input_state(app), c),
    );
}

fn handle_backspace(app: &mut App) {
    apply_ai_input(app, text_input::backspace(&ai_input_state(app)));
}

fn handle_delete(app: &mut App) {
    apply_ai_input(app, text_input::delete_forward(&ai_input_state(app)));
}

fn handle_cursor_left(app: &mut App, extend_selection: bool) {
    apply_ai_input(
        app,
        text_input::cursor_left(&ai_input_state(app), extend_selection),
    );
}

fn handle_cursor_right(app: &mut App, extend_selection: bool) {
    apply_ai_input(
        app,
        text_input::cursor_right(&ai_input_state(app), extend_selection),
    );
}

fn handle_tab_in_input(app: &mut App) {
    apply_ai_input(
        app,
        text_input::insert_tab(&ai_input_state(app)),
    );
}

fn main() {
    load_env_files();

    let _ = terminal_input::enter_raw_mode();

    let win = initscr();
    noecho();
    curs_set(0);
    init_ui_colors();

    let mut app = App::new();
    draw_ui(&win, &app);

    while !app.quit {
        app.tick();

        let key = terminal_input::poll_key(Duration::from_millis(POLL_MS as u64))
            .ok()
            .flatten();
        match key {
            Some(TerminalKey::Tab) => {
                if ai_input_editing(&app) {
                    handle_tab_in_input(&mut app);
                } else {
                    app.focus = app.focus.next(app.show_clear_response);
                }
            }
            Some(TerminalKey::Escape) => app.quit = true,
            Some(TerminalKey::Quit) if app.focus != Focus::AiInput => {
                app.quit = true;
            }
            Some(TerminalKey::Enter) => {
                if !(app.focus == Focus::AiInput && !app.ai_input_locked) {
                    app.activate_focused();
                }
            }
            Some(TerminalKey::Space) => {
                if app.focus == Focus::AiInput && !app.ai_input_locked {
                    handle_input_char(&mut app, ' ');
                } else {
                    app.activate_focused();
                }
            }
            Some(TerminalKey::Backspace) => handle_backspace(&mut app),
            Some(TerminalKey::Delete) => handle_delete(&mut app),
            Some(TerminalKey::Left { extend_selection }) => {
                if ai_input_editing(&app) {
                    handle_cursor_left(&mut app, extend_selection);
                } else {
                    app.focus = app.focus.prev(app.show_clear_response);
                }
            }
            Some(TerminalKey::Right { extend_selection }) => {
                if ai_input_editing(&app) {
                    handle_cursor_right(&mut app, extend_selection);
                } else {
                    app.focus = app.focus.next(app.show_clear_response);
                }
            }
            Some(TerminalKey::Up) => {
                app.focus = app.focus.prev(app.show_clear_response);
            }
            Some(TerminalKey::Down) => {
                app.focus = app.focus.next(app.show_clear_response);
            }
            Some(TerminalKey::Char(c)) => handle_input_char(&mut app, c),
            Some(TerminalKey::Quit) | None => {}
        }

        draw_ui(&win, &app);
    }

    curs_set(1);
    echo();
    endwin();
    let _ = terminal_input::leave_raw_mode();
}
