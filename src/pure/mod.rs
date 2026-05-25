//! Side-effect-free helpers used by the runtime (no I/O, no UI).

pub(crate) mod element_id;
pub(crate) mod element_placement;
pub(crate) mod focus_key;
pub(crate) mod focus_order;
pub(crate) mod focus_store;
pub(crate) mod keyboard;
pub(crate) mod layout_reflow;
pub(crate) mod message_gutter;
pub(crate) mod resize_debounce;
pub(crate) mod screen_scroll;
pub(crate) mod scroll_view;
pub(crate) mod terminal_bounds;
pub(crate) mod terminal_input_batch;
pub(crate) mod text_input;
pub(crate) mod text_wrap;
pub(crate) mod ui_redraw;
