//! Minimal terminal curses UI with Gemini streaming test.

mod ai_output;
mod backend;
mod clipboard;
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
    CLEAR_RESP_BTN_WIDTH, COL_BTN, COL_FLASH_TEXT, COL_TITLE,
    PAIR_TEXT_INPUT_SELECT, ROW_FIRST_BTN,
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
    AiOutput,
}

impl Focus {
    fn next(self, show_clear: bool, show_output: bool) -> Self {
        match self {
            Focus::Foo => Focus::Bar,
            Focus::Bar => Focus::AiInput,
            Focus::AiInput => Focus::TestAi,
            Focus::TestAi => {
                if show_clear {
                    Focus::ClearResponse
                } else if show_output {
                    Focus::AiOutput
                } else {
                    Focus::Foo
                }
            }
            Focus::ClearResponse => {
                if show_output {
                    Focus::AiOutput
                } else {
                    Focus::Foo
                }
            }
            Focus::AiOutput => Focus::Foo,
        }
    }

    fn prev(self, show_clear: bool, show_output: bool) -> Self {
        match self {
            Focus::Foo => {
                if show_output {
                    Focus::AiOutput
                } else if show_clear {
                    Focus::ClearResponse
                } else {
                    Focus::TestAi
                }
            }
            Focus::AiOutput => {
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
}

fn compute_layout(foo_flash: bool, bar_flash: bool, ai_input_rows: i32) -> Layout {
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
    ai_response_scroll: usize,
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
            ai_response_scroll: 0,
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
        compute_layout(
            self.foo_flash.is_some(),
            self.bar_flash.is_some(),
            ai_input_rows,
        )
    }

    fn ai_output_viewport(&self, win: &pancurses::Window) -> ai_output::AiOutputViewport {
        ai_output::AiOutputViewport::from_window(win, self.layout().ai_response_y)
    }

    fn ai_output_shown(&self) -> bool {
        self.ai_waiting_for_first_token() || !self.ai_response.is_empty()
    }

    fn focus_nav(&self) -> (bool, bool) {
        (self.show_clear_response, self.ai_output_shown())
    }

    fn ai_output_hovered(&self) -> bool {
        self.focus == Focus::AiOutput
    }

    fn ai_output_content_overflows(&self, win: &pancurses::Window) -> bool {
        if self.ai_waiting_for_first_token() || !self.ai_output_shown() {
            return false;
        }
        let viewport = self.ai_output_viewport(win);
        ai_output::content_overflows(&self.ai_response, viewport)
    }

    fn ai_output_scrollable(&self, win: &pancurses::Window) -> bool {
        self.ai_output_hovered() && self.ai_output_content_overflows(win)
    }

    fn sync_ai_output_scroll(&mut self, win: &pancurses::Window) {
        if !self.ai_output_shown() || self.ai_waiting_for_first_token() {
            self.ai_response_scroll = 0;
            return;
        }
        if !self.ai_output_hovered() {
            return;
        }
        let viewport = self.ai_output_viewport(win);
        self.ai_response_scroll = if self.ai_streaming {
            ai_output::stick_to_bottom(&self.ai_response, viewport)
        } else {
            ai_output::clamp_scroll(self.ai_response_scroll, &self.ai_response, viewport)
        };
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
        self.ai_response_scroll = 0;
        self.show_clear_response = false;
        self.ai_streaming = true;
        self.last_ai_request_at = Some(Instant::now());
        self.ai_rx = Some(start_stream(&key, &self.ai_input));
    }

    fn clear_ai_response(&mut self) {
        self.ai_response.clear();
        self.ai_response_scroll = 0;
        self.show_clear_response = false;
        self.ai_input_locked = false;
        self.ai_input_selection_anchor = None;
        if matches!(self.focus, Focus::ClearResponse | Focus::AiOutput) {
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
            Focus::AiInput | Focus::AiOutput => {}
        }
    }

    fn tick(&mut self) {
        tick_flash(&mut self.foo_flash);
        tick_flash(&mut self.bar_flash);
        self.poll_ai();
        if self.focus == Focus::AiOutput && !self.ai_output_shown() {
            self.focus = Focus::TestAi;
        }
    }
}

fn tick_flash(slot: &mut Option<Instant>) {
    if let Some(at) = *slot {
        if at.elapsed() >= FLASH_SECS {
            *slot = None;
        }
    }
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
    let highlight_cells = text_wrap::selection_highlight_cells(text, selection, width);

    let mut char_index = 0usize;
    let mut drawn_cells = std::collections::BTreeSet::new();
    for ch in text.chars() {
        let (line, col) = text_wrap::cursor_display_position(text, char_index, width);
        if ch != '\n' {
            let selected = highlight_cells.contains(&(line, col));
            let ch_pair = if selected {
                PAIR_TEXT_INPUT_SELECT
            } else {
                pair
            };
            win.attron(COLOR_PAIR(ch_pair));
            win.mv(y + line as i32, COL_BTN + col as i32);
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
        win.attron(COLOR_PAIR(PAIR_TEXT_INPUT_SELECT));
        win.mv(y + line as i32, COL_BTN + col as i32);
        win.addch(' ');
        win.attroff(COLOR_PAIR(PAIR_TEXT_INPUT_SELECT));
    }
}

fn draw_ui(win: &pancurses::Window, app: &App) {
    let layout = app.layout();
    let output_viewport = app.ai_output_viewport(win);

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

    let output_focused = app.focus == Focus::AiOutput;
    if app.ai_waiting_for_first_token() {
        ai_output::draw_line(
            win,
            layout.ai_response_y,
            AI_RES_WAITING_TEXT,
            output_viewport,
            output_focused,
        );
    } else if !app.ai_response.is_empty() {
        ai_output::draw_scrollable(
            win,
            &app.ai_response,
            app.ai_response_scroll,
            output_viewport,
            output_focused,
        );
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

fn handle_enter_in_input(app: &mut App) {
    apply_ai_input(
        app,
        text_input::insert_newline(&ai_input_state(app)),
    );
}

fn handle_copy(app: &mut App) {
    if let Some((state, text)) = text_input::copy_selection(&ai_input_state(app)) {
        if clipboard::set_text(&text) {
            set_ai_input_state(app, state);
        }
    }
}

fn handle_cut(app: &mut App) {
    if let Some((state, text)) = text_input::cut_selection(&ai_input_state(app)) {
        if clipboard::set_text(&text) {
            set_ai_input_state(app, state);
        }
    }
}

fn handle_paste(app: &mut App) {
    if let Some(paste) = clipboard::get_text() {
        apply_ai_input(app, text_input::paste_text(&ai_input_state(app), &paste));
    }
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
        app.sync_ai_output_scroll(&win);

        let key = terminal_input::poll_key(Duration::from_millis(POLL_MS as u64))
            .ok()
            .flatten();
        match key {
            Some(TerminalKey::Tab) => {
                if ai_input_editing(&app) {
                    handle_tab_in_input(&mut app);
                } else {
                    let nav = app.focus_nav();
                    app.focus = app.focus.next(nav.0, nav.1);
                }
            }
            Some(TerminalKey::Escape) => app.quit = true,
            Some(TerminalKey::Quit) if app.focus != Focus::AiInput => {
                app.quit = true;
            }
            Some(TerminalKey::Enter) => {
                if ai_input_editing(&app) {
                    handle_enter_in_input(&mut app);
                } else {
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
            Some(TerminalKey::Backspace) if ai_input_editing(&app) => handle_backspace(&mut app),
            Some(TerminalKey::Delete) if ai_input_editing(&app) => handle_delete(&mut app),
            Some(TerminalKey::Left { extend_selection }) => {
                if ai_input_editing(&app) {
                    handle_cursor_left(&mut app, extend_selection);
                } else {
                    let nav = app.focus_nav();
                    app.focus = app.focus.prev(nav.0, nav.1);
                }
            }
            Some(TerminalKey::Right { extend_selection }) => {
                if ai_input_editing(&app) {
                    handle_cursor_right(&mut app, extend_selection);
                } else {
                    let nav = app.focus_nav();
                    app.focus = app.focus.next(nav.0, nav.1);
                }
            }
            Some(TerminalKey::Up) => {
                let nav = app.focus_nav();
                app.focus = app.focus.prev(nav.0, nav.1);
            }
            Some(TerminalKey::Down) => {
                let nav = app.focus_nav();
                app.focus = app.focus.next(nav.0, nav.1);
            }
            Some(TerminalKey::AltUp)
                if app.ai_output_scrollable(&win) && app.ai_response_scroll > 0 =>
            {
                app.ai_response_scroll = ai_output::scroll_up(app.ai_response_scroll);
            }
            Some(TerminalKey::AltDown) if app.ai_output_scrollable(&win) => {
                let viewport = app.ai_output_viewport(&win);
                app.ai_response_scroll = ai_output::scroll_down(
                    app.ai_response_scroll,
                    &app.ai_response,
                    viewport,
                );
            }
            Some(TerminalKey::Copy) if ai_input_editing(&app) => handle_copy(&mut app),
            Some(TerminalKey::Cut) if ai_input_editing(&app) => handle_cut(&mut app),
            Some(TerminalKey::Paste) if ai_input_editing(&app) => handle_paste(&mut app),
            Some(TerminalKey::Char(c)) if ai_input_editing(&app) => handle_input_char(&mut app, c),
            Some(TerminalKey::Backspace)
            | Some(TerminalKey::Delete)
            | Some(TerminalKey::Char(_))
            | Some(TerminalKey::Copy)
            | Some(TerminalKey::Cut)
            | Some(TerminalKey::Paste)
            | Some(TerminalKey::Quit)
            | Some(TerminalKey::AltUp)
            | Some(TerminalKey::AltDown)
            | None => {}
        }

        draw_ui(&win, &app);
    }

    curs_set(1);
    echo();
    endwin();
    let _ = terminal_input::leave_raw_mode();
}
