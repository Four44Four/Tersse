//! UI layout, size, color, and API constants.

use pancurses::{
    can_change_color, has_colors, init_color, init_pair, start_color, use_default_colors, COLORS,
};

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

// --- Text colors (pair foreground / background) ---

/// Locked AI prompt text: custom `#999999`.
pub const AI_INPUT_TEXT_HEX: &str = "#999999";
/// Curses palette index where `#999999` is registered via `init_color`.
pub const AI_INPUT_TEXT_COLOR: i16 = 16;
pub const AI_INPUT_TEXT_BG: i16 = -1;

pub const AI_RES_TEXT_COLOR: i16 = pancurses::COLOR_WHITE;
pub const AI_RES_TEXT_BG: i16 = -1;

/// Shown as AI response text when **Test AI** is used with an empty prompt.
pub const AI_NO_PROMPT_RES_TEXT: &str = "Please enter a prompt";

// --- Color pairs ---

pub const PAIR_BTN_NORMAL: u64 = 1;
pub const PAIR_BTN_FOCUSED: u64 = 2;
pub const PAIR_AI_INPUT: u64 = 3;
pub const PAIR_AI_RESPONSE: u64 = 4;

pub const BTN_NORMAL_FG: i16 = pancurses::COLOR_WHITE;
pub const BTN_NORMAL_BG: i16 = pancurses::COLOR_BLUE;
pub const BTN_FOCUSED_FG: i16 = pancurses::COLOR_BLACK;
pub const BTN_FOCUSED_BG: i16 = pancurses::COLOR_CYAN;

/// Convert an 8-bit channel (0–255) to curses `init_color` scale (0–1000).
const fn channel_1000(c: u8) -> i16 {
    ((c as i32) * 1000 / 255) as i16
}

/// Parse `#RRGGBB` into `(r, g, b)` bytes.
fn hex_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
    (r, g, b)
}

/// Register `#999999` in the curses palette; returns the foreground index for `init_pair`.
fn register_ai_input_text_color() -> i16 {
    let (r, g, b) = hex_rgb(AI_INPUT_TEXT_HEX);
    let index = if (COLORS() as i16) > AI_INPUT_TEXT_COLOR {
        AI_INPUT_TEXT_COLOR
    } else {
        pancurses::COLOR_YELLOW
    };

    if can_change_color() && index == AI_INPUT_TEXT_COLOR {
        init_color(
            index,
            channel_1000(r),
            channel_1000(g),
            channel_1000(b),
        );
        index
    } else {
        pancurses::COLOR_YELLOW
    }
}

/// Register all curses color pairs used by the UI.
pub fn init_ui_colors() {
    if has_colors() {
        use_default_colors();
        start_color();
        init_pair(PAIR_BTN_NORMAL as i16, BTN_NORMAL_FG, BTN_NORMAL_BG);
        init_pair(PAIR_BTN_FOCUSED as i16, BTN_FOCUSED_FG, BTN_FOCUSED_BG);

        let ai_input_fg = register_ai_input_text_color();
        init_pair(PAIR_AI_INPUT as i16, ai_input_fg, AI_INPUT_TEXT_BG);

        init_pair(
            PAIR_AI_RESPONSE as i16,
            AI_RES_TEXT_COLOR,
            AI_RES_TEXT_BG,
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
