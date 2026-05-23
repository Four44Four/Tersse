use tersse::runtime::{FocusStyle, Style, TextInputStyle};
use tersse::{set_title_of_current_screen, Color, ScreenTitle, TitleAlignment};

pub fn screen_title() -> ScreenTitle {
    set_title_of_current_screen(
        "Hello world",
        TitleAlignment::Left,
        Color::White,
        Color::Black,
    )
}

pub fn button_style() -> FocusStyle {
    FocusStyle {
        focused: Style {
            fg: Color::Black,
            bg: Color::Cyan,
        },
        unfocused: Style {
            fg: Color::White,
            bg: Color::Blue,
        },
    }
}

pub fn locked_like_style() -> FocusStyle {
    FocusStyle {
        focused: Style {
            fg: Color::Red,
            bg: Color::White,
        },
        unfocused: Style {
            fg: Color::Yellow,
            bg: Color::Black,
        },
    }
}

// pub fn neutral_display_style() -> FocusStyle {
//     FocusStyle {
//         focused: Style {
//             fg: Color::White,
//             bg: Color::Black,
//         },
//         unfocused: Style {
//             fg: Color::White,
//             bg: Color::Black,
//         },
//     }
// }

pub fn text_input_style() -> TextInputStyle {
    TextInputStyle {
        focused_unlocked: Style {
            fg: Color::Black,
            bg: Color::White,
        },
        unfocused_unlocked: Style {
            fg: Color::White,
            bg: Color::Default,
        },
        focused_locked: Style {
            fg: Color::Red,
            bg: Color::White,
        },
        unfocused_locked: Style {
            fg: Color::Yellow,
            bg: Color::Default,
        },
        selection: Style {
            fg: Color::White,
            bg: Color::Black,
        },
    }
}
