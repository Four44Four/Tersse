mod style;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tersse::prelude::*;

use style::{button_style, locked_like_style, screen_title, text_input_style};

const POLL_TIMEOUT_MS: u64 = 50;
const FLASH_FOO: Duration = Duration::from_secs(2);
const FLASH_BAR: Duration = Duration::from_secs(5);
const FOO_BAR_BUTTON_WIDTH: usize = 5;
const INPUT_WIDTH: usize = 20;
const PRESS_LABEL: &str = "Press Me !!";
const CLEAR_LABEL: &str = "Clear Result";
const DISPLAY_WIDTH: usize = 80;
const RESULT_HEIGHT: usize = 12;

#[derive(Clone, Copy)]
struct FlashMessage {
    expires_at: Instant,
}

struct App {
    foo_id: ElementId,
    bar_id: ElementId,
    input_id: ElementId,
    press_id: ElementId,
    clear_id: Option<ElementId>,
    result_id: Option<ElementId>,
    foo_text_id: Option<ElementId>,
    bar_text_id: Option<ElementId>,
    foo_flash: Option<FlashMessage>,
    bar_flash: Option<FlashMessage>,
    result_visible: bool,
}

impl App {
    fn new(foo_id: ElementId, bar_id: ElementId, input_id: ElementId, press_id: ElementId) -> Self {
        Self {
            foo_id,
            bar_id,
            input_id,
            press_id,
            clear_id: None,
            result_id: None,
            foo_text_id: None,
            bar_text_id: None,
            foo_flash: None,
            bar_flash: None,
            result_visible: false,
        }
    }

    fn tick(&mut self, ui: &mut RuntimeUi) {
        if Self::message_expired(self.foo_flash) {
            self.foo_flash = None;
            if let Some(id) = self.foo_text_id.take() {
                let _ = ui.remove_and_reflow(id);
            }
        }
        if Self::message_expired(self.bar_flash) {
            self.bar_flash = None;
            if let Some(id) = self.bar_text_id.take() {
                let _ = ui.remove_and_reflow(id);
            }
        }
    }

    fn handle_foo(&mut self, ui: &mut RuntimeUi) {
        self.foo_flash = Some(FlashMessage {
            expires_at: Instant::now() + FLASH_FOO,
        });
        self.foo_text_id = Some(self.upsert_flash_display(
            ui,
            self.foo_id,
            self.foo_text_id,
            0.5,
            "Button 1",
        ));
    }

    fn handle_bar(&mut self, ui: &mut RuntimeUi) {
        self.bar_flash = Some(FlashMessage {
            expires_at: Instant::now() + FLASH_BAR,
        });
        self.bar_text_id = Some(self.upsert_flash_display(
            ui,
            self.bar_id,
            self.bar_text_id,
            1.5,
            "Button 2",
        ));
    }

    fn handle_press(&mut self, ui: &mut RuntimeUi, app_state: Rc<RefCell<Option<App>>>) {
        let input = ui.read_text_input(self.input_id).unwrap_or_default();
        let result = build_result_text(&input);
        let _ = ui.set_text_input_lock_status(self.input_id, true);

        let clear_x = PRESS_LABEL.chars().count().max(1) + 1;

        if self.clear_id.is_none() {
            self.clear_id = Some(ui.create_button(ButtonConfig {
                label: CLEAR_LABEL.to_string(),
                width: CLEAR_LABEL.chars().count().max(1),
                placement: ElementPlacement::relative_to(
                    self.input_id,
                    ParentSide::Bottom,
                    Location {
                        x: clear_x as u16,
                        y: 0,
                    },
                ),
                focus_number: 4.0,
                style: button_style(),
                on_press: Box::new(move |ui| {
                    app_state
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .handle_clear(ui);
                }),
            }));
        }

        if self.result_visible {
            if let Some(result_id) = self.result_id {
                let _ = ui.set_text_display_text(result_id, result);
            }
        } else {
            self.result_id = Some(ui.create_text_display(TextDisplayConfig {
                placement: ElementPlacement::relative_to(
                    self.press_id,
                    ParentSide::Bottom,
                    Location::default(),
                ),
                width: DISPLAY_WIDTH,
                height: RESULT_HEIGHT,
                focus_number: 5.0,
                style: locked_like_style(),
                initial_text: result,
            }));
            self.result_visible = true;
        }
    }

    fn handle_clear(&mut self, ui: &mut RuntimeUi) {
        if let Some(result_id) = self.result_id.take() {
            let _ = ui.remove_and_reflow(result_id);
        }
        if let Some(clear_id) = self.clear_id.take() {
            let _ = ui.remove_element(clear_id);
        }
        let _ = ui.set_text_input_lock_status(self.input_id, false);
        self.result_visible = false;
    }

    fn upsert_flash_display(
        &self,
        ui: &mut RuntimeUi,
        button_id: ElementId,
        display_id: Option<ElementId>,
        focus_number: f64,
        text: &str,
    ) -> ElementId {
        let width = text.chars().count().max(1);
        let config = TextDisplayConfig {
            placement: ElementPlacement::relative_to(
                button_id,
                ParentSide::Bottom,
                Location::default(),
            ),
            width,
            height: 1,
            focus_number,
            style: locked_like_style(),
            initial_text: text.to_string(),
        };
        if let Some(id) = display_id {
            let _ = ui.update_text_display(id, config);
            id
        } else {
            ui.create_text_display(config)
        }
    }

    fn message_expired(message: Option<FlashMessage>) -> bool {
        matches!(message, Some(msg) if Instant::now() >= msg.expires_at)
    }
}

fn main() {
    let mut ui = RuntimeUi::new();
    ui.set_title(screen_title());

    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

    let foo_app = Rc::clone(&app);
    let foo_id = ui.create_button(ButtonConfig {
        label: "Foo".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        placement: ElementPlacement::absolute(Location { x: 0, y: 2 }),
        focus_number: 0.0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            foo_app.borrow_mut().as_mut().unwrap().handle_foo(ui);
        }),
    });

    let bar_app = Rc::clone(&app);
    let bar_id = ui.create_button(ButtonConfig {
        label: "Bar".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        placement: ElementPlacement::absolute(Location { x: 0, y: 3 }),
        focus_number: 1.0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            bar_app.borrow_mut().as_mut().unwrap().handle_bar(ui);
        }),
    });

    let input_id = ui.create_text_input(TextInputConfig {
        width: INPUT_WIDTH,
        placement: ElementPlacement::absolute(Location { x: 0, y: 4 }),
        focus_number: 2.0,
        style: text_input_style(),
        locked: false,
        initial_text: String::new(),
    });

    let press_app = Rc::clone(&app);
    let press_id = ui.create_button(ButtonConfig {
        label: PRESS_LABEL.to_string(),
        width: PRESS_LABEL.chars().count().max(1),
        placement: ElementPlacement::relative_to(input_id, ParentSide::Bottom, Location::default()),
        focus_number: 3.0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            press_app
                .borrow_mut()
                .as_mut()
                .unwrap()
                .handle_press(ui, Rc::clone(&press_app));
        }),
    });

    *app.borrow_mut() = Some(App::new(foo_id, bar_id, input_id, press_id));

    loop {
        {
            let mut slot = app.borrow_mut();
            if let Some(app) = slot.as_mut() {
                app.tick(&mut ui);
            }
        }
        if !ui.run_frame(Duration::from_millis(POLL_TIMEOUT_MS)) {
            break;
        }
    }
}

fn build_result_text(input: &str) -> String {
    let reversed = input.chars().rev().collect::<String>();
    reversed.repeat(10)
}
