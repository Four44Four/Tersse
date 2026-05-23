//! Fixed-width button label layout (truncate / pad).

/// Truncate `label` to at most `width` columns.
pub fn truncate_label(label: &str, width: usize) -> String {
    label.chars().take(width.max(1)).collect()
}

/// Space columns needed after `label` so the button occupies `width` columns.
pub fn padding_cols(label: &str, width: usize) -> usize {
    let width = width.max(1);
    width.saturating_sub(label.chars().count().min(width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_label() {
        assert_eq!(truncate_label("abcdef", 3), "abc");
    }

    #[test]
    fn padding_fills_short_label() {
        assert_eq!(padding_cols("ab", 5), 3);
        assert_eq!(padding_cols("hello", 5), 0);
    }
}
