//! System clipboard read/write (platform I/O).

use arboard::Clipboard;

pub fn set_text(text: &str) -> bool {
    Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .is_ok()
}

pub fn get_text() -> Option<String> {
    Clipboard::new().ok()?.get_text().ok()
}
