//! Minimal terminal curses UI with Gemini streaming test.

mod backend;
mod constants;

use backend::{start_stream, AiStreamEvent};
use constants::{
    button_color_pair, clear_resp_btn_x, gemini_api_key, init_ui_colors, load_env_files,
    text_input_color_pair, ALLOW_MOUSE_INPUT, AI_INPUT_HEIGHT, AI_INPUT_WIDTH,
    AI_MISSING_API_KEY_RES_TEXT,
    AI_NO_PROMPT_RES_TEXT, BTN_HEIGHT, BTN_WIDTH, CLEAR_RESP_BTN_HEIGHT, CLEAR_RESP_BTN_LABEL,
    CLEAR_RESP_BTN_WIDTH, COL_BTN, COL_FLASH_TEXT, COL_TITLE, PAIR_AI_RESPONSE, ROW_FIRST_BTN,
    ROW_TITLE, TEST_AI_BTN_HEIGHT, TEST_AI_BTN_LABEL, TEST_AI_BTN_WIDTH,
};
use pancurses::{echo, endwin, initscr, noecho, Input};
use pancurses::{curs_set, COLOR_PAIR};
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
    const ALL: [DemoButton; 2] = [DemoButton::Foo, DemoButton::Bar];

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

    fn index(self) -> usize {
        match self {
            DemoButton::Foo => 0,
            DemoButton::Bar => 1,
        }
    }

    fn from_index(i: usize) -> DemoButton {
        Self::ALL[i % 2]
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

fn compute_layout(foo_flash: bool, bar_flash: bool, ai_response_rows: i32) -> Layout {
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
    y += AI_INPUT_HEIGHT;

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

impl Layout {
    fn hit_demo_button(self, row: i32, col: i32) -> Option<DemoButton> {
        if col < COL_BTN || col >= COL_BTN + BTN_WIDTH {
            return None;
        }
        if row >= self.foo_y && row < self.foo_y + BTN_HEIGHT {
            return Some(DemoButton::Foo);
        }
        if row >= self.bar_y && row < self.bar_y + BTN_HEIGHT {
            return Some(DemoButton::Bar);
        }
        None
    }

    fn hit_ai_input(self, row: i32, col: i32) -> bool {
        row == self.ai_input_y && col >= COL_BTN && col < COL_BTN + AI_INPUT_WIDTH
    }

    fn hit_test_ai(self, row: i32, col: i32) -> bool {
        row == self.test_ai_y
            && col >= COL_BTN
            && col < COL_BTN + TEST_AI_BTN_WIDTH
    }

    fn hit_clear_response(self, row: i32, col: i32) -> bool {
        let x = clear_resp_btn_x();
        row == self.test_ai_y
            && col >= x
            && col < x + CLEAR_RESP_BTN_WIDTH
    }
}

struct App {
    focus: Focus,
    foo_flash: Option<Instant>,
    bar_flash: Option<Instant>,
    ai_input: String,
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
            ai_input_locked: false,
            ai_response: String::new(),
            ai_streaming: false,
            ai_rx: None,
            show_clear_response: false,
            last_ai_request_at: None,
            quit: false,
        }
    }

    fn layout(&self) -> Layout {
        let rows = wrapped_line_count(&self.ai_response, text_wrap_width());
        compute_layout(
            self.foo_flash.is_some(),
            self.bar_flash.is_some(),
            rows,
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
            Focus::AiInput => self.start_ai_request(),
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

fn wrapped_lines(text: &str, width: i32) -> Vec<String> {
    let width = width.max(1) as usize;
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
            continue;
        }
        current.push(ch);
        if current.chars().count() >= width {
            lines.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn wrapped_line_count(text: &str, width: i32) -> i32 {
    wrapped_lines(text, width).len() as i32
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

fn draw_ai_input(win: &pancurses::Window, y: i32, text: &str, locked: bool, focused: bool) {
    let display: String = text.chars().take(AI_INPUT_WIDTH as usize).collect();
    let pair = text_input_color_pair(focused, locked);
    fill_solid(win, y, COL_BTN, AI_INPUT_WIDTH, AI_INPUT_HEIGHT, pair);
    win.attron(COLOR_PAIR(pair));
    win.mv(y, COL_BTN);
    win.addstr(&display);
    win.attroff(COLOR_PAIR(pair));
}

fn draw_ai_response(win: &pancurses::Window, y: i32, text: &str) {
    win.attron(COLOR_PAIR(PAIR_AI_RESPONSE));
    for (i, line) in wrapped_lines(text, text_wrap_width()).iter().enumerate() {
        win.mv(y + i as i32, COL_BTN);
        win.addstr(line);
    }
    win.attroff(COLOR_PAIR(PAIR_AI_RESPONSE));
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

    if !app.ai_response.is_empty() {
        draw_ai_response(win, layout.ai_response_y, &app.ai_response);
    }

    if app.focus == Focus::AiInput && !app.ai_input_locked {
        let cursor_x = COL_BTN + app.ai_input.chars().count() as i32;
        curs_set(1);
        win.mv(layout.ai_input_y, cursor_x.min(COL_BTN + AI_INPUT_WIDTH - 1));
    } else {
        curs_set(0);
    }

    win.refresh();
}

fn handle_input_char(app: &mut App, c: char) {
    if app.focus != Focus::AiInput || app.ai_input_locked {
        return;
    }
    if c.is_control() {
        return;
    }
    if app.ai_input.chars().count() < AI_INPUT_WIDTH as usize {
        app.ai_input.push(c);
    }
}

fn handle_backspace(app: &mut App) {
    if app.focus != Focus::AiInput || app.ai_input_locked {
        return;
    }
    app.ai_input.pop();
}

fn apply_mouse_focus(app: &mut App, layout: Layout, row: i32, col: i32) {
    if let Some(btn) = layout.hit_demo_button(row, col) {
        app.focus = match btn {
            DemoButton::Foo => Focus::Foo,
            DemoButton::Bar => Focus::Bar,
        };
        return;
    }
    if layout.hit_ai_input(row, col) {
        app.focus = Focus::AiInput;
        return;
    }
    if layout.hit_test_ai(row, col) {
        app.focus = Focus::TestAi;
        return;
    }
    if app.show_clear_response && layout.hit_clear_response(row, col) {
        app.focus = Focus::ClearResponse;
    }
}

fn main() {
    load_env_files();

    let win = initscr();
    noecho();
    curs_set(0);
    win.keypad(true);
    win.timeout(POLL_MS);

    if ALLOW_MOUSE_INPUT {
        let _ = pancurses::mousemask(pancurses::BUTTON1_CLICKED, None);
    }
    init_ui_colors();

    let mut app = App::new();
    draw_ui(&win, &app);

    while !app.quit {
        app.tick();

        match win.getch() {
            Some(Input::Character('\t')) => {
                app.focus = app.focus.next(app.show_clear_response);
            }
            Some(Input::Character(c)) if matches!(c, 'q' | 'Q') && app.focus != Focus::AiInput => {
                app.quit = true;
            }
            Some(Input::Character('\x1b')) => app.quit = true,
            Some(Input::Character(c)) if matches!(c, '\n' | '\r' | ' ') => app.activate_focused(),
            Some(Input::Character(c)) if c == '\x08' || c == '\x7f' => handle_backspace(&mut app),
            Some(Input::Character(c)) => handle_input_char(&mut app, c),
            Some(Input::KeyBackspace) => handle_backspace(&mut app),
            Some(Input::KeyUp) | Some(Input::KeyLeft) => {
                app.focus = app.focus.prev(app.show_clear_response);
            }
            Some(Input::KeyDown) | Some(Input::KeyRight) => {
                app.focus = app.focus.next(app.show_clear_response);
            }
            Some(Input::KeyMouse) if ALLOW_MOUSE_INPUT => {
                if let Ok(evt) = pancurses::getmouse() {
                    if evt.bstate & pancurses::BUTTON1_CLICKED != 0 {
                        let layout = app.layout();
                        apply_mouse_focus(&mut app, layout, evt.y, evt.x);
                        app.activate_focused();
                    }
                }
            }
            Some(_) | None => {}
        }

        draw_ui(&win, &app);
    }

    curs_set(1);
    echo();
    endwin();
}
