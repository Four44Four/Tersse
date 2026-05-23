use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tersse::runtime::{
    ButtonConfig, ElementConfig, FocusStyle, RuntimeUi, Style, TextDisplayConfig, TextInputConfig,
    TextInputStyle,
};
use tersse::{set_title_of_current_screen, Color, Location, TitleAlignment};

const POLL_TIMEOUT_MS: u64 = 50;
const FLASH_FOO: Duration = Duration::from_secs(2);
const FLASH_BAR: Duration = Duration::from_secs(5);
const FOO_BAR_BUTTON_WIDTH: usize = 5;
const INPUT_WIDTH: usize = 20;
const PRESS_LABEL: &str = "Press Me !!";
const CLEAR_LABEL: &str = "Clear Result";
const DISPLAY_WIDTH: usize = 80;
const RESULT_HEIGHT: usize = 12;

const FOO_ID: &str = "foo_button";
const FOO_TEXT_ID: &str = "foo_text_display";
const BAR_ID: &str = "bar_button";
const BAR_TEXT_ID: &str = "bar_text_display";
const INPUT_ID: &str = "input";
const PRESS_ID: &str = "press_button";
const CLEAR_ID: &str = "clear_button";
const RESULT_ID: &str = "result_display";

#[derive(Clone, Copy)]
struct FlashMessage {
    expires_at: Instant,
}

struct App {
    foo_flash: Option<FlashMessage>,
    bar_flash: Option<FlashMessage>,
    result_visible: bool,
}

impl App {
    fn new() -> Self {
        Self {
            foo_flash: None,
            bar_flash: None,
            result_visible: false,
        }
    }

    fn tick(&mut self, ui: &mut RuntimeUi) {
        if Self::message_expired(self.foo_flash) {
            self.foo_flash = None;
            let _ = ui.remove_and_reflow(FOO_TEXT_ID);
        }
        if Self::message_expired(self.bar_flash) {
            self.bar_flash = None;
            let _ = ui.remove_and_reflow(BAR_TEXT_ID);
        }
    }

    fn handle_foo(&mut self, ui: &mut RuntimeUi) {
        self.foo_flash = Some(FlashMessage {
            expires_at: Instant::now() + FLASH_FOO,
        });
        self.upsert_flash_display(ui, FOO_ID, FOO_TEXT_ID, "Button 1");
    }

    fn handle_bar(&mut self, ui: &mut RuntimeUi) {
        self.bar_flash = Some(FlashMessage {
            expires_at: Instant::now() + FLASH_BAR,
        });
        self.upsert_flash_display(ui, BAR_ID, BAR_TEXT_ID, "Button 2");
    }

    fn handle_press(&mut self, ui: &mut RuntimeUi, app_state: Rc<RefCell<App>>) {
        let input = ui.read_text_input(INPUT_ID).unwrap_or_default();
        let result = build_result_text(&input);
        let _ = ui.set_text_input_lock_status(INPUT_ID, true);

        let press_loc = ui
            .element_location(PRESS_ID)
            .unwrap_or(Location { x: 0, y: 5 });
        let press_width = ui.button_width(PRESS_ID).unwrap_or(label_width(PRESS_LABEL));
        let clear_x = press_loc.x as usize + press_width + 1;

        ui.upsert_button(ButtonConfig {
            id: CLEAR_ID.to_string(),
            label: CLEAR_LABEL.to_string(),
            width: label_width(CLEAR_LABEL),
            location: Location {
                x: clear_x as u16,
                y: press_loc.y,
            },
            focus_index: 4,
            style: button_style(),
            on_press: Box::new(move |ui| {
                let mut app = app_state.borrow_mut();
                app.handle_clear(ui);
            }),
        });

        if self.result_visible {
            let _ = ui.set_text_display_text(RESULT_ID, result);
        } else {
            ui.upsert_and_reflow(ElementConfig::TextDisplay(TextDisplayConfig {
                id: RESULT_ID.to_string(),
                location: Location {
                    x: 0,
                    y: press_loc.y.saturating_add(1),
                },
                width: DISPLAY_WIDTH,
                height: RESULT_HEIGHT,
                focus_index: 5,
                style: locked_like_style(),
                initial_text: result,
            }));
            self.result_visible = true;
        }
    }

    fn handle_clear(&mut self, ui: &mut RuntimeUi) {
        let _ = ui.remove_and_reflow(RESULT_ID);
        let _ = ui.remove_element(CLEAR_ID);
        let _ = ui.set_text_input_lock_status(INPUT_ID, false);
        self.result_visible = false;
    }

    fn upsert_flash_display(
        &self,
        ui: &mut RuntimeUi,
        button_id: &str,
        display_id: &str,
        text: &str,
    ) {
        let y = ui
            .element_location(button_id)
            .map(|loc| loc.y.saturating_add(1))
            .unwrap_or(3);
        let width = text.chars().count().max(1);
        ui.upsert_and_reflow(ElementConfig::TextDisplay(TextDisplayConfig {
            id: display_id.to_string(),
            location: Location { x: 0, y },
            width,
            height: 1,
            focus_index: 99,
            style: neutral_display_style(),
            initial_text: text.to_string(),
        }));
    }

    fn message_expired(message: Option<FlashMessage>) -> bool {
        matches!(message, Some(msg) if Instant::now() >= msg.expires_at)
    }
}

fn main() {
    let mut ui = RuntimeUi::new();
    let app = Rc::new(RefCell::new(App::new()));
    ui.set_title(set_title_of_current_screen(
        "Hello world",
        TitleAlignment::Left,
        Color::rgb(255, 255, 255),
        Color::rgb(0, 0, 0),
    ));

    let foo_app = Rc::clone(&app);
    ui.upsert_button(ButtonConfig {
        id: FOO_ID.to_string(),
        label: "Foo".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        location: Location { x: 0, y: 2 },
        focus_index: 0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            let mut app = foo_app.borrow_mut();
            app.handle_foo(ui);
        }),
    });
    let bar_app = Rc::clone(&app);
    ui.upsert_button(ButtonConfig {
        id: BAR_ID.to_string(),
        label: "Bar".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        location: Location { x: 0, y: 3 },
        focus_index: 1,
        style: button_style(),
        on_press: Box::new(move |ui| {
            let mut app = bar_app.borrow_mut();
            app.handle_bar(ui);
        }),
    });
    ui.upsert_text_input(TextInputConfig {
        id: INPUT_ID.to_string(),
        width: INPUT_WIDTH,
        location: Location { x: 0, y: 4 },
        focus_index: 2,
        style: text_input_style(),
        locked: false,
        initial_text: String::new(),
    });
    let press_app = Rc::clone(&app);
    ui.upsert_button(ButtonConfig {
        id: PRESS_ID.to_string(),
        label: PRESS_LABEL.to_string(),
        width: label_width(PRESS_LABEL),
        location: Location { x: 0, y: 5 },
        focus_index: 3,
        style: button_style(),
        on_press: Box::new(move |ui| {
            let mut app = press_app.borrow_mut();
            app.handle_press(ui, Rc::clone(&press_app));
        }),
    });
    loop {
        {
            let mut app = app.borrow_mut();
            app.tick(&mut ui);
        }
        if !ui.run_frame(Duration::from_millis(POLL_TIMEOUT_MS)) {
            break;
        }
    }
}

fn label_width(label: &str) -> usize {
    label.chars().count().max(1)
}

fn build_result_text(input: &str) -> String {
    let reversed = input.chars().rev().collect::<String>();
    reversed.repeat(10)
}

fn button_style() -> FocusStyle {
    FocusStyle {
        focused: Style {
            fg: Color::rgb(0, 0, 0),
            bg: Color::rgb(0, 255, 255),
        },
        unfocused: Style {
            fg: Color::rgb(255, 255, 255),
            bg: Color::rgb(0, 0, 255),
        },
    }
}

fn locked_like_style() -> FocusStyle {
    FocusStyle {
        focused: Style {
            fg: Color::rgb(255, 0, 0),
            bg: Color::rgb(255, 255, 255),
        },
        unfocused: Style {
            fg: Color::rgb(255, 255, 0),
            bg: Color::rgb(0, 0, 0),
        },
    }
}

fn neutral_display_style() -> FocusStyle {
    FocusStyle {
        focused: Style {
            fg: Color::rgb(255, 255, 255),
            bg: Color::rgb(0, 0, 0),
        },
        unfocused: Style {
            fg: Color::rgb(255, 255, 255),
            bg: Color::rgb(0, 0, 0),
        },
    }
}

fn text_input_style() -> TextInputStyle {
    TextInputStyle {
        focused_unlocked: Style {
            fg: Color::rgb(0, 0, 0),
            bg: Color::rgb(255, 255, 255),
        },
        unfocused_unlocked: Style {
            fg: Color::rgb(255, 255, 255),
            bg: Color::rgb(0, 0, 0),
        },
        focused_locked: Style {
            fg: Color::rgb(255, 0, 0),
            bg: Color::rgb(255, 255, 255),
        },
        unfocused_locked: Style {
            fg: Color::rgb(255, 255, 0),
            bg: Color::rgb(0, 0, 0),
        },
        selection: Style {
            fg: Color::rgb(255, 255, 255),
            bg: Color::rgb(0, 0, 0),
        },
    }
}