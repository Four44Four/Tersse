//! Coalesce terminal poll items produced in a single read burst (e.g. paste on WinAPI).

/// A single keyboard/paste item before coalescing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalPollItem {
    Paste(String),
    Char(char),
    Tab,
    Space,
    Enter,
    Other,
}

/// Merge consecutive text-bearing items into [`TerminalPollItem::Paste`] runs.
pub fn coalesce_terminal_poll_items(items: Vec<TerminalPollItem>) -> Vec<TerminalPollItem> {
    let mut out = Vec::new();
    let mut text = String::new();

    let flush = |out: &mut Vec<TerminalPollItem>, text: &mut String| {
        if !text.is_empty() {
            out.push(TerminalPollItem::Paste(std::mem::take(text)));
        }
    };

    for item in items {
        match item {
            TerminalPollItem::Paste(s) => text.push_str(&s),
            TerminalPollItem::Char(c) if coalesce_char(c) => text.push(c),
            TerminalPollItem::Tab if !text.is_empty() => text.push('\t'),
            TerminalPollItem::Space if !text.is_empty() => text.push(' '),
            TerminalPollItem::Enter if !text.is_empty() => text.push('\n'),
            other => {
                flush(&mut out, &mut text);
                out.push(other);
            }
        }
    }
    flush(&mut out, &mut text);
    out
}

fn coalesce_char(c: char) -> bool {
    !c.is_control() || c == '\t'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesce_char_run_into_single_paste() {
        let items = vec![
            TerminalPollItem::Char('a'),
            TerminalPollItem::Char('b'),
            TerminalPollItem::Char('c'),
        ];
        assert_eq!(
            coalesce_terminal_poll_items(items),
            vec![TerminalPollItem::Paste("abc".to_string())]
        );
    }

    #[test]
    fn coalesce_merges_existing_paste_with_following_chars() {
        let items = vec![
            TerminalPollItem::Paste("x".to_string()),
            TerminalPollItem::Char('y'),
        ];
        assert_eq!(
            coalesce_terminal_poll_items(items),
            vec![TerminalPollItem::Paste("xy".to_string())]
        );
    }

    #[test]
    fn coalesce_flushes_before_non_text_item() {
        let items = vec![
            TerminalPollItem::Char('a'),
            TerminalPollItem::Other,
            TerminalPollItem::Char('b'),
        ];
        assert_eq!(
            coalesce_terminal_poll_items(items),
            vec![
                TerminalPollItem::Paste("a".to_string()),
                TerminalPollItem::Other,
                TerminalPollItem::Paste("b".to_string()),
            ]
        );
    }

    #[test]
    fn coalesce_includes_whitespace_and_newlines() {
        let items = vec![
            TerminalPollItem::Char('a'),
            TerminalPollItem::Space,
            TerminalPollItem::Tab,
            TerminalPollItem::Enter,
            TerminalPollItem::Char('b'),
        ];
        assert_eq!(
            coalesce_terminal_poll_items(items),
            vec![TerminalPollItem::Paste("a \t\nb".to_string())]
        );
    }
}
