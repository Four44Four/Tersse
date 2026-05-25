//! Test-only API surface. Not part of the public crate API for library users.
//!
//! Enable with the `test-api` Cargo feature (also enables `pure-tests` for `tersse::pure`).

pub use crate::constants::UI_REDRAW_DEBOUNCE_QUEUE_UPDATE_MS;
pub use crate::element_id::ElementId;
pub use crate::runtime::{
    runtime_clamp_fixed_height, runtime_render_height_for_element_text, runtime_terminal_color_code,
};
