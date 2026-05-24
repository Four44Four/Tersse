use ahash::AHashMap;
use std::time::Instant;

use pancurses::Window;

use crate::ScreenTitle;

mod colors;
mod core;
#[cfg(debug_draw_do_delay)]
mod debug_draw_delay;
mod element_store;
mod elements;
mod events;
mod focus;
mod layout;
mod message_gutter;
mod placement;
mod render;
mod resize;
mod screen_scroll;
mod text_input_state;
mod types;
mod ui_session;

pub use types::{
    ElementConfig, ElementHandler, ElementHeightMode, FocusStyle, Style, TextInputBehavior,
    TextInputStyle,
};
pub use ui_session::UiSession;

/// Cloneable handle to the shared Tokio runtime used by [`RuntimeUi`].
///
/// Use this to spawn async background tasks without creating a separate runtime.
#[derive(Clone, Debug)]
pub struct UiRuntime {
    handle: tokio::runtime::Handle,
}

impl UiRuntime {
    pub(crate) fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }

    /// Spawns a future on the shared runtime.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.handle.spawn(future)
    }
}

use crate::pure::message_gutter::MessageGutterState;

use element_store::ElementStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextInputLayoutCache {
    text_len: usize,
    width: usize,
    height: usize,
}

pub fn runtime_terminal_color_code(color: crate::Color) -> i16 {
    colors::terminal_color_code(color)
}

pub fn runtime_render_height_for_element_text(text: &str, width: usize) -> usize {
    layout::render_height_for_text_input_text(text, width)
}

pub fn runtime_clamp_fixed_height(height: usize) -> usize {
    types::clamp_fixed_height(height)
}

pub fn runtime_text_input_state_snapshot(
    text: impl Into<String>,
    cursor: usize,
    selection_anchor: Option<usize>,
) -> crate::pure::text_input::TextInputState {
    types::text_input_state_from_parts(text, cursor, selection_anchor)
}

pub struct RuntimeUi {
    win: Window,
    title: Option<ScreenTitle>,
    elements: ElementStore,
    focused_position: usize,
    pair_cache: AHashMap<(i16, i16), i16>,
    next_pair_id: i16,
    next_element_id: usize,
    cached_heights: AHashMap<usize, usize>,
    text_input_layout_cache: AHashMap<usize, TextInputLayoutCache>,
    resize_debounce_until: Option<Instant>,
    redraw_debounce_until: Option<Instant>,
    last_terminal_yx: Option<(i32, i32)>,
    screen_scroll: usize,
    ui_queue: ui_session::UiQueue,
    ui_signal_tx: ui_session::UiSignalSender,
    ui_signal_rx: ui_session::UiSignalReceiver,
    async_driver: Option<core::AsyncRuntimeDriver>,
    has_rendered_first_frame: bool,
    ui_queue_redraw_pending: bool,
    ui_queue_redraw_plan: crate::pure::ui_redraw::ElementRedrawPlan,
    draining_ui_queue: bool,
    sync_layout_redraw_pending: bool,
    message_gutter: MessageGutterState,
    message_gutter_expires_at: Option<Instant>,
}
