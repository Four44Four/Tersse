//! UI layout, size, color, and API constants.

use pancurses::{has_colors, init_pair, start_color, use_default_colors};

// --- API (Gemini-compatible REST) ---

/// Default model; override with `GEMINI_MODEL` in env (e.g. `gemini-2.5-flash-lite`).
pub const GEMINI_MODEL: &str = "gemini-2.5-flash";
pub const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Shown when `GEMINI_API_KEY` is not set in the environment.
pub const AI_MISSING_API_KEY_RES_TEXT: &str =
    "GEMINI_API_KEY environment variable is not set";

/// Load env files: `.env`, then `.env.development` (debug) or `.env.production` (release).
/// Mode-specific files override values from `.env`.
pub fn load_env_files() {
    let _ = dotenvy::dotenv();
    #[cfg(debug_assertions)]
    let _ = dotenvy::from_filename_override(".env.development");
    #[cfg(not(debug_assertions))]
    let _ = dotenvy::from_filename_override(".env.production");
}

/// Read the Gemini API key from the `GEMINI_API_KEY` environment variable.
pub fn gemini_api_key() -> Result<String, std::env::VarError> {
    std::env::var("GEMINI_API_KEY").map(|k| k.trim().to_string())
}

/// Model id for `streamGenerateContent` (env `GEMINI_MODEL` or [`GEMINI_MODEL`] default).
pub fn gemini_model() -> String {
    std::env::var("GEMINI_MODEL")
        .map(|m| m.trim().to_string())
        .unwrap_or_else(|_| GEMINI_MODEL.to_string())
}

// --- Positions ---

pub const ROW_TITLE: i32 = 0;
pub const COL_TITLE: i32 = 0;

pub const ROW_FIRST_BTN: i32 = 2;
pub const COL_BTN: i32 = 0;
pub const COL_FLASH_TEXT: i32 = 0;

// --- Sizes (Foo / Bar) ---

pub const BTN_WIDTH: i32 = 6;
pub const BTN_HEIGHT: i32 = 1;

// --- AI input / buttons ---

pub const AI_INPUT_WIDTH: i32 = 48;
pub const AI_INPUT_HEIGHT: i32 = 1;

pub const TEST_AI_BTN_WIDTH: i32 = 8;
pub const TEST_AI_BTN_HEIGHT: i32 = 1;
pub const TEST_AI_BTN_LABEL: &str = "Test AI";

pub const CLEAR_RESP_BTN_WIDTH: i32 = 15;
pub const CLEAR_RESP_BTN_HEIGHT: i32 = 1;
pub const CLEAR_RESP_BTN_LABEL: &str = "Clear Response";
pub const CLEAR_RESP_BTN_GAP: i32 = 1;

// --- Text input colors (editing, unlocked) ---

pub const TEXT_INPUT_NON_HOVERED_COLOR_FG: i16 = pancurses::COLOR_WHITE;
pub const TEXT_INPUT_NON_HOVERED_COLOR_BG: i16 = -1;

pub const TEXT_INPUT_HOVERED_COLOR_FG: i16 = pancurses::COLOR_BLACK;
pub const TEXT_INPUT_HOVERED_COLOR_BG: i16 = pancurses::COLOR_WHITE;

// --- Text input colors (locked / submitted prompt) ---

pub const TEXT_INPUT_LOCKED_NON_HOVERED_COLOR_FG: i16 = pancurses::COLOR_YELLOW;
pub const TEXT_INPUT_LOCKED_NON_HOVERED_COLOR_BG: i16 = -1;

pub const TEXT_INPUT_LOCKED_HOVERED_COLOR_FG: i16 = pancurses::COLOR_RED;
pub const TEXT_INPUT_LOCKED_HOVERED_COLOR_BG: i16 = pancurses::COLOR_WHITE;

pub const TEXT_INPUT_SELECT_FG_COLOR: i16 = pancurses::COLOR_WHITE;
pub const TEXT_INPUT_SELECT_BG_COLOR: i16 = pancurses::COLOR_BLACK;

/// Shown as AI response text when **Test AI** is used with an empty prompt.
pub const AI_NO_PROMPT_RES_TEXT: &str = "Please enter a prompt";

/// Shown below **Test AI** (yellow) while waiting for the first model token.
pub const AI_RES_WAITING_TEXT: &str = "Waiting for response...";

// --- Color pairs ---

pub const PAIR_BTN_NORMAL: u64 = 1;
pub const PAIR_BTN_FOCUSED: u64 = 2;
pub const PAIR_TEXT_INPUT_NON_HOVERED: u64 = 3;
pub const PAIR_TEXT_INPUT_HOVERED: u64 = 4;
pub const PAIR_TEXT_INPUT_LOCKED_NON_HOVERED: u64 = 5;
pub const PAIR_TEXT_INPUT_LOCKED_FOCUSED: u64 = 6;
pub const PAIR_TEXT_INPUT_SELECT: u64 = 7;

pub const BTN_NORMAL_FG: i16 = pancurses::COLOR_WHITE;
pub const BTN_NORMAL_BG: i16 = pancurses::COLOR_BLUE;
pub const BTN_FOCUSED_FG: i16 = pancurses::COLOR_BLACK;
pub const BTN_FOCUSED_BG: i16 = pancurses::COLOR_CYAN;

/// Color pair for AI output (same locked-input colors, including hover background).
pub fn ai_output_color_pair(focused: bool) -> u64 {
    text_input_color_pair(focused, true)
}

/// Color pair for the text input in the given focus / lock state.
pub fn text_input_color_pair(focused: bool, locked: bool) -> u64 {
    if locked {
        if focused {
            PAIR_TEXT_INPUT_LOCKED_FOCUSED
        } else {
            PAIR_TEXT_INPUT_LOCKED_NON_HOVERED
        }
    } else if focused {
        PAIR_TEXT_INPUT_HOVERED
    } else {
        PAIR_TEXT_INPUT_NON_HOVERED
    }
}

/// Register all curses color pairs used by the UI.
pub fn init_ui_colors() {
    if has_colors() {
        use_default_colors();
        start_color();
        init_pair(PAIR_BTN_NORMAL as i16, BTN_NORMAL_FG, BTN_NORMAL_BG);
        init_pair(PAIR_BTN_FOCUSED as i16, BTN_FOCUSED_FG, BTN_FOCUSED_BG);

        init_pair(
            PAIR_TEXT_INPUT_NON_HOVERED as i16,
            TEXT_INPUT_NON_HOVERED_COLOR_FG,
            TEXT_INPUT_NON_HOVERED_COLOR_BG,
        );
        init_pair(
            PAIR_TEXT_INPUT_HOVERED as i16,
            TEXT_INPUT_HOVERED_COLOR_FG,
            TEXT_INPUT_HOVERED_COLOR_BG,
        );
        init_pair(
            PAIR_TEXT_INPUT_LOCKED_NON_HOVERED as i16,
            TEXT_INPUT_LOCKED_NON_HOVERED_COLOR_FG,
            TEXT_INPUT_LOCKED_NON_HOVERED_COLOR_BG,
        );
        init_pair(
            PAIR_TEXT_INPUT_LOCKED_FOCUSED as i16,
            TEXT_INPUT_LOCKED_HOVERED_COLOR_FG,
            TEXT_INPUT_LOCKED_HOVERED_COLOR_BG,
        );
        init_pair(
            PAIR_TEXT_INPUT_SELECT as i16,
            TEXT_INPUT_SELECT_FG_COLOR,
            TEXT_INPUT_SELECT_BG_COLOR,
        );
    }
}

/// Color pair for a standard action button in the given focus state.
pub fn button_color_pair(focused: bool) -> u64 {
    if focused {
        PAIR_BTN_FOCUSED
    } else {
        PAIR_BTN_NORMAL
    }
}

pub fn clear_resp_btn_x() -> i32 {
    COL_BTN + TEST_AI_BTN_WIDTH + CLEAR_RESP_BTN_GAP
}
