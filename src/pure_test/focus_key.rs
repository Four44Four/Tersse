use std::cmp::Ordering;

/// Sort key for focus-ordered TUI elements: `(focus_number, id)`.
#[derive(Clone, Debug)]
pub struct FocusKey {
    pub focus_number: f64,
    pub id: usize,
}

impl PartialEq for FocusKey {
    fn eq(&self, other: &Self) -> bool {
        self.focus_number.to_bits() == other.focus_number.to_bits() && self.id == other.id
    }
}

impl Eq for FocusKey {}

impl FocusKey {
    pub fn new(focus_number: f64, id: usize) -> Self {
        Self { focus_number, id }
    }
}

impl PartialOrd for FocusKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FocusKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.focus_number
            .total_cmp(&other.focus_number)
            .then_with(|| self.id.cmp(&other.id))
    }
}
