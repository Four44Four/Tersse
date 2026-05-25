//! Runtime pending-paste buffer (mirrors `RuntimeUi::pending_text`).

/// Accumulates paste chunks until a boundary or batch end, matching runtime `pending_text`.
#[derive(Debug, Default)]
pub struct PendingPasteBuffer {
    pub(crate) text: String,
}

impl PendingPasteBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg_attr(not(feature = "pure-tests"), allow(dead_code))]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[cfg_attr(not(feature = "pure-tests"), allow(dead_code))]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn push_paste(&mut self, paste: &str) {
        self.text.push_str(paste);
    }

    /// Flush before resize or key handling.
    pub fn flush_before_boundary(&mut self) -> Option<String> {
        if self.text.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.text))
        }
    }

    #[cfg_attr(not(feature = "pure-tests"), allow(dead_code))]
    pub fn flush_end(&mut self) -> Option<String> {
        self.flush_before_boundary()
    }
}
