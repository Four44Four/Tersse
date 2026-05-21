//! Minimal terminal curses UI: "Hello world" and Foo/Bar buttons.

use pancurses::{echo, endwin, has_colors, init_pair, initscr, noecho, start_color, Input};
use pancurses::{curs_set, COLOR_PAIR};
use std::time::{Duration, Instant};

const FLASH_SECS: Duration = Duration::from_secs(2);
const POLL_MS: i32 = 50;

const ROW_TITLE: i32 = 0;
const ROW_FIRST_BTN: i32 = 2;

const BTN_W: i32 = 6;
const BTN_H: i32 = 1;
const BTN_X: i32 = 0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Button {
    Foo,
    Bar,
}

impl Button {
    const ALL: [Button; 2] = [Button::Foo, Button::Bar];

    fn label(self) -> &'static str {
        match self {
            Button::Foo => "Foo",
            Button::Bar => "Bar",
        }
    }

    fn index(self) -> usize {
        match self {
            Button::Foo => 0,
            Button::Bar => 1,
        }
    }

    fn from_index(i: usize) -> Button {
        Self::ALL[i % 2]
    }
}

struct Layout {
    foo_y: i32,
    bar_y: i32,
    flash_y: Option<i32>,
}

/// Stack buttons with no gap; insert a flash row under the active button.
fn compute_layout(flash: Option<Button>) -> Layout {
    let mut y = ROW_FIRST_BTN;

    let foo_y = y;
    y += BTN_H;

    if flash == Some(Button::Foo) {
        let flash_y = y;
        y += BTN_H;
        return Layout {
            foo_y,
            bar_y: y,
            flash_y: Some(flash_y),
        };
    }

    let bar_y = y;
    y += BTN_H;

    Layout {
        foo_y,
        bar_y,
        flash_y: flash.filter(|&b| b == Button::Bar).map(|_| y),
    }
}

impl Layout {
    fn hit_button(self, row: i32, col: i32) -> Option<Button> {
        if col < BTN_X || col >= BTN_X + BTN_W {
            return None;
        }
        if row >= self.foo_y && row < self.foo_y + BTN_H {
            return Some(Button::Foo);
        }
        if row >= self.bar_y && row < self.bar_y + BTN_H {
            return Some(Button::Bar);
        }
        None
    }
}

struct App {
    focus: Button,
    flash: Option<(Button, Instant)>,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            focus: Button::Foo,
            flash: None,
            quit: false,
        }
    }

    fn activate(&mut self, button: Button) {
        self.flash = Some((button, Instant::now()));
    }

    fn tick(&mut self) {
        if let Some((_, at)) = self.flash {
            if at.elapsed() >= FLASH_SECS {
                self.flash = None;
            }
        }
    }

    fn flash_button(&self) -> Option<Button> {
        self.flash.map(|(b, _)| b)
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

fn draw_button(win: &pancurses::Window, y: i32, button: Button, focused: bool) {
    let pair = if focused { 2 } else { 1 };
    fill_solid(win, y, BTN_X, BTN_W, BTN_H, pair);
    win.attron(COLOR_PAIR(pair));
    win.mv(y, BTN_X);
    win.addstr(button.label());
    win.attroff(COLOR_PAIR(pair));
}

fn draw_ui(win: &pancurses::Window, app: &App) {
    let layout = compute_layout(app.flash_button());

    win.erase();

    win.mv(ROW_TITLE, 0);
    win.addstr("Hello world");

    draw_button(win, layout.foo_y, Button::Foo, app.focus == Button::Foo);
    draw_button(win, layout.bar_y, Button::Bar, app.focus == Button::Bar);

    if let (Some(btn), Some(flash_y)) = (app.flash_button(), layout.flash_y) {
        win.mv(flash_y, BTN_X);
        win.addstr(btn.label());
    }

    win.refresh();
}

fn main() {
    let win = initscr();
    noecho();
    curs_set(0);
    win.keypad(true);
    win.timeout(POLL_MS);

    let _ = pancurses::mousemask(pancurses::BUTTON1_CLICKED, None);

    if has_colors() {
        start_color();
        init_pair(1, pancurses::COLOR_WHITE, pancurses::COLOR_BLUE);
        init_pair(2, pancurses::COLOR_BLACK, pancurses::COLOR_CYAN);
    }

    let mut app = App::new();
    draw_ui(&win, &app);

    while !app.quit {
        app.tick();

        match win.getch() {
            Some(Input::Character('\t')) => {
                app.focus = Button::from_index(app.focus.index() + 1);
            }
            Some(Input::Character(c)) if matches!(c, 'q' | 'Q' | '\x1b') => app.quit = true,
            Some(Input::Character(c)) if matches!(c, '\n' | '\r' | ' ') => {
                app.activate(app.focus);
            }
            Some(Input::KeyUp) | Some(Input::KeyLeft) => {
                app.focus = Button::from_index(app.focus.index().wrapping_sub(1));
            }
            Some(Input::KeyDown) | Some(Input::KeyRight) => {
                app.focus = Button::from_index(app.focus.index() + 1);
            }
            Some(Input::KeyMouse) => {
                if let Ok(evt) = pancurses::getmouse() {
                    let layout = compute_layout(app.flash_button());
                    if let Some(btn) = layout.hit_button(evt.y, evt.x) {
                        app.focus = btn;
                        app.activate(btn);
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
