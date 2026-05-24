mod style;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::time::sleep;
use tersse::prelude::*;

use style::{button_style, locked_like_style, screen_title, text_input_style};

const FLASH_FOO: Duration = Duration::from_secs(2);
const FLASH_BAR: Duration = Duration::from_secs(5);
const FOO_BAR_BUTTON_WIDTH: usize = 5;
const MUNG_BUTTON_WIDTH: usize = 5;
const MUNG_BAR_MARGIN: u16 = 3;
const INPUT_WIDTH: usize = 20;
const PRESS_LABEL: &str = "Press Me !!";
const CLEAR_LABEL: &str = "Clear Result";
const DISPLAY_WIDTH: usize = 80;
const RESULT_HEIGHT: usize = 12;

struct App {
    foo_id: ElementId,
    bar_id: ElementId,
    input_id: ElementId,
    press_id: ElementId,
    clear_id: Option<ElementId>,
    result_id: Option<ElementId>,
    foo_text_id: Arc<Mutex<Option<ElementId>>>,
    bar_text_id: Arc<Mutex<Option<ElementId>>>,
    foo_flash: Option<JoinHandle<()>>,
    bar_flash: Option<JoinHandle<()>>,
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
            foo_text_id: Arc::new(Mutex::new(None)),
            bar_text_id: Arc::new(Mutex::new(None)),
            foo_flash: None,
            bar_flash: None,
            result_visible: false,
        }
    }

    fn handle_foo(&mut self, ui: &mut RuntimeUi, runtime: UiRuntime, session: &UiSession) {
        self.foo_flash.take().inspect(|task| task.abort());
        let id = self.upsert_flash_display(
            ui,
            self.foo_id,
            *self.foo_text_id.lock().unwrap(),
            0.5,
            "Button 1",
        );
        *self.foo_text_id.lock().unwrap() = Some(id);
        self.foo_flash = Some(schedule_flash_removal(
            runtime,
            session.clone(),
            Arc::clone(&self.foo_text_id),
            FLASH_FOO,
        ));
    }

    fn handle_bar(&mut self, ui: &mut RuntimeUi, runtime: UiRuntime, session: &UiSession) {
        self.bar_flash.take().inspect(|task| task.abort());
        let id = self.upsert_flash_display(
            ui,
            self.bar_id,
            *self.bar_text_id.lock().unwrap(),
            1.5,
            "Button 2",
        );
        *self.bar_text_id.lock().unwrap() = Some(id);
        self.bar_flash = Some(schedule_flash_removal(
            runtime,
            session.clone(),
            Arc::clone(&self.bar_text_id),
            FLASH_BAR,
        ));
    }

    fn handle_press(&mut self, ui: &mut RuntimeUi, app: Rc<RefCell<Option<App>>>) {
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
                    app.borrow_mut().as_mut().unwrap().handle_clear(ui);
                }),
            }));
        }

        if !self.result_visible {
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
        let placement = ElementPlacement::relative_to(
            button_id,
            ParentSide::Bottom,
            Location::default(),
        );
        if !display_id.is_none() {
            return display_id.expect("what");
        }
        ui.create_text_display(TextDisplayConfig {
            placement,
            width,
            height: 1,
            focus_number,
            style: locked_like_style(),
            initial_text: text.to_string(),
        })
    }
}

fn schedule_flash_removal(
    runtime: UiRuntime,
    session: UiSession,
    id_slot: Arc<Mutex<Option<ElementId>>>,
    after: Duration,
) -> JoinHandle<()> {
    runtime.spawn(async move {
        sleep(after).await;
        session.queue_update(move |ui| {
            if let Some(element_id) = id_slot.lock().unwrap().take() {
                let _ = ui.remove_and_reflow(element_id);
            }
        });
    })
}

fn main() {
    let mut ui = RuntimeUi::new();
    let runtime = ui.runtime();
    let session = ui.ui_session();
    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

    ui.set_title(screen_title());

    let foo_app = Rc::clone(&app);
    let foo_runtime = runtime.clone();
    let foo_session = session.clone();
    let foo_id = ui.create_button(ButtonConfig {
        label: "Foo".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        placement: ElementPlacement::absolute(Location { x: 0, y: 2 }),
        focus_number: 0.0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            foo_app
                .borrow_mut()
                .as_mut()
                .unwrap()
                .handle_foo(ui, foo_runtime.clone(), &foo_session);
        }),
    });

    let bar_app = Rc::clone(&app);
    let bar_runtime = runtime.clone();
    let bar_session = session.clone();
    let bar_id = ui.create_button(ButtonConfig {
        label: "Bar".to_string(),
        width: FOO_BAR_BUTTON_WIDTH,
        placement: ElementPlacement::absolute(Location { x: 0, y: 3 }),
        focus_number: 1.0,
        style: button_style(),
        on_press: Box::new(move |ui| {
            bar_app
                .borrow_mut()
                .as_mut()
                .unwrap()
                .handle_bar(ui, bar_runtime.clone(), &bar_session);
        }),
    });

    let mung_session = session.clone();
    let _mung_id = ui.create_button(ButtonConfig {
        label: "Mung".to_string(),
        width: MUNG_BUTTON_WIDTH,
        placement: ElementPlacement::relative_to(
            bar_id,
            ParentSide::Right,
            Location {
                x: MUNG_BAR_MARGIN,
                y: 0,
            },
        ),
        focus_number: 1.5,
        style: button_style(),
        on_press: Box::new(move |_ui| {
            mung_session.send_message(format!("{}What ?", random_base64_chars(5)));
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

    app.borrow_mut()
        .replace(App::new(foo_id, bar_id, input_id, press_id));

    ui.run();
}

fn build_result_text(input: &str) -> String {
    let reversed = input.chars().rev().collect::<String>();
    reversed.repeat(10)
}

fn random_base64_chars(count: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            CHARSET[(state as usize) % CHARSET.len()] as char
        })
        .collect()
}
