//! Pure helpers for keyboard-driven text-input behavior.

/// Whether a horizontal arrow press should extend the selection (Shift held).
pub fn arrow_extend_selection(shift_pressed: bool) -> bool {
    shift_pressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_extends_selection() {
        assert!(arrow_extend_selection(true));
        assert!(!arrow_extend_selection(false));
    }
}
