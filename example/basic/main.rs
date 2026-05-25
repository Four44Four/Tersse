mod style;

use std::cell::RefCell;
use std::rc::Rc;

use crossterm::terminal::size;
use tersse::prelude::*;

use style::{button_style, text_display_style, text_element_style, text_input_style};

const TITLE: &str = "Very cool";
const FOO_LABEL: &str = "Foo";
const BOO_LABEL: &str = "Boo.";
const SUBMIT_LABEL: &str = "Submit";
const FOO_WIDTH: usize = 5;
const INPUT_WIDTH: usize = 20;
const INPUT_HEIGHT: usize = 2;
const BUTTON_HEIGHT: usize = 1;
const BOO_MARGIN: u16 = 1;
const FOO_Y: u16 = 2;

struct App {
    foo_id: ElementId,
    boo_id: Option<ElementId>,
    input_id: ElementId,
}

impl App {
    fn handle_foo(&mut self, ui: &mut RuntimeUi) {
        if let Some(boo_id) = self.boo_id.take() {
            let _ = ui.remove_and_reflow(boo_id);
            return;
        }
        self.boo_id = Some(ui.create_element(static_text_display_unfocusable_fit_width(
            ElementPlacement::relative_to(
                self.foo_id,
                ParentSide::Right,
                Location {
                    x: BOO_MARGIN,
                    y: 0,
                },
            ),
            text_display_style(),
            BOO_LABEL,
        )));
    }

    fn handle_submit(&self, ui: &mut RuntimeUi, session: &UiSession) {
        let message = ui.read_element_text(self.input_id).unwrap_or_default();
        session.send_message(message);
    }
}

fn centered_x(text: &str, cols: u16) -> u16 {
    let width = text.chars().count();
    cols.saturating_sub(width as u16) / 2
}

fn main() {
    let (cols, _rows) = size().unwrap_or((80, 24));
    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

    let mut ui = RuntimeUi::new();
    let session = ui.ui_session();

    let _title_id = ui.create_element(static_text_display_unfocusable_fit_width(
        ElementPlacement::absolute(Location {
            x: centered_x(TITLE, cols),
            y: 0,
        }),
        text_display_style(),
        TITLE,
    ));

    let foo_app = Rc::clone(&app);
    let foo_id = ui.create_element(button(
        ElementPlacement::absolute(Location { x: 0, y: FOO_Y }),
        FOO_WIDTH,
        BUTTON_HEIGHT,
        0.0,
        button_style(),
        FOO_LABEL,
        Box::new(move |ui| {
            foo_app.borrow_mut().as_mut().unwrap().handle_foo(ui);
        }),
    ));

    let input_id = ui.create_element(text_input_fixed(
        ElementPlacement::relative_to(foo_id, ParentSide::Bottom, Location::default()),
        INPUT_WIDTH,
        INPUT_HEIGHT,
        1.0,
        text_element_style(),
        text_input_style(),
        String::new(),
        false,
    ));

    let submit_app = Rc::clone(&app);
    let submit_session = session.clone();
    let _submit_id = ui.create_element(button_fit_width(
        ElementPlacement::relative_to(input_id, ParentSide::Bottom, Location::default()),
        BUTTON_HEIGHT,
        2.0,
        button_style(),
        SUBMIT_LABEL,
        Box::new(move |ui| {
            submit_app
                .borrow()
                .as_ref()
                .unwrap()
                .handle_submit(ui, &submit_session);
        }),
    ));

    app.borrow_mut().replace(App {
        foo_id,
        boo_id: None,
        input_id,
    });

    ui.run();
}
